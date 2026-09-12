package controlplane

import (
	"context"
	"crypto/tls"
	"fmt"
	"net"
	"net/http"
	"os"
	"sync/atomic"
	"time"

	discovery "github.com/dubml/xds-api/service/discovery/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/keepalive"
	"k8s.io/client-go/dynamic"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
)

type ServerOptions struct {
	Kubernetes      *rest.Config
	XDSAddress      string
	HealthAddress   string
	CertificateFile string
	PrivateKeyFile  string
	ClusterDomain   string
	RootCAFile      string
	Deployment      DeploymentOptions
}

func Run(ctx context.Context, options ServerOptions) error {
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()
	if options.Kubernetes == nil {
		return fmt.Errorf("Kubernetes client configuration is required")
	}
	if options.CertificateFile == "" || options.PrivateKeyFile == "" {
		return fmt.Errorf("xDS requires a TLS certificate and private key")
	}
	rootCA, err := os.ReadFile(options.RootCAFile)
	if err != nil {
		return fmt.Errorf("load xDS root CA for data planes: %w", err)
	}
	options.Deployment.RootCA = string(rootCA)
	if err := options.Deployment.Validate(); err != nil {
		return err
	}
	loadCertificate := func(*tls.ClientHelloInfo) (*tls.Certificate, error) {
		cert, err := tls.LoadX509KeyPair(options.CertificateFile, options.PrivateKeyFile)
		if err != nil {
			return nil, fmt.Errorf("load xDS server certificate: %w", err)
		}
		return &cert, nil
	}
	if _, err := loadCertificate(nil); err != nil {
		return err
	}
	client, err := kubernetes.NewForConfig(options.Kubernetes)
	if err != nil {
		return err
	}
	dynamicClient, err := dynamic.NewForConfig(options.Kubernetes)
	if err != nil {
		return err
	}
	source, err := NewKubernetesSource(dynamicClient, ctx.Done())
	if err != nil {
		return err
	}
	if err := source.CheckAccess(ctx); err != nil {
		return err
	}
	if options.ClusterDomain == "" {
		options.ClusterDomain = "cluster.local"
	}
	outputs := NewOutputs(source.Inputs, ControllerName, options.ClusterDomain, ctx.Done())
	discoveryServer := NewXDSServer(outputs, TokenAuthorizer(client.AuthenticationV1()))
	reconciler, err := NewReconciler(dynamicClient, source.Inputs, outputs, options.Deployment, discoveryServer)
	if err != nil {
		return err
	}
	defer reconciler.queue.ShutDown()
	if err := reconciler.CheckAccess(ctx); err != nil {
		return err
	}
	server := grpc.NewServer(
		grpc.Creds(credentials.NewTLS(&tls.Config{MinVersion: tls.VersionTLS12, GetCertificate: loadCertificate})),
		grpc.MaxRecvMsgSize(4*1024*1024),
		grpc.MaxConcurrentStreams(4),
		grpc.KeepaliveParams(keepalive.ServerParameters{MaxConnectionAge: 10 * time.Minute, MaxConnectionAgeGrace: time.Minute}),
	)
	discovery.RegisterAggregatedDiscoveryServiceServer(server, discoveryServer)
	listener, err := net.Listen("tcp", options.XDSAddress)
	if err != nil {
		return fmt.Errorf("listen for xDS: %w", err)
	}
	defer listener.Close()
	var ready atomic.Bool
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusOK) })
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, _ *http.Request) {
		if !ready.Load() {
			http.Error(w, "initial resource synchronization is incomplete", http.StatusServiceUnavailable)
			return
		}
		w.WriteHeader(http.StatusOK)
	})
	mux.HandleFunc("/metrics", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/plain; version=0.0.4")
		fmt.Fprintf(w, "# TYPE transit_xds_streams gauge\ntransit_xds_streams %d\n# TYPE transit_xds_acks_total counter\ntransit_xds_acks_total %d\n# TYPE transit_xds_nacks_total counter\ntransit_xds_nacks_total %d\n", discoveryServer.Connections.Load(), discoveryServer.ACKs.Load(), discoveryServer.NACKs.Load())
	})
	health := &http.Server{Addr: options.HealthAddress, Handler: mux, ReadHeaderTimeout: 5 * time.Second, ReadTimeout: 10 * time.Second, WriteTimeout: 10 * time.Second, IdleTimeout: 30 * time.Second}
	errCh := make(chan error, 3)
	go func() {
		if err := health.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			errCh <- err
		}
	}()
	defer func() {
		shutdown, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = health.Shutdown(shutdown)
	}()
	defer func() {
		ready.Store(false)
		done := make(chan struct{})
		go func() { server.GracefulStop(); close(done) }()
		select {
		case <-done:
		case <-time.After(5 * time.Second):
			server.Stop()
		}
	}()
	go func() { errCh <- server.Serve(listener) }()
	go func() { errCh <- reconciler.Run(ctx) }()
	synchronized := make(chan error, 1)
	go func() {
		if err := source.Start(ctx); err != nil {
			synchronized <- err
			return
		}
		if !outputs.WaitUntilSynced(ctx.Done()) {
			synchronized <- fmt.Errorf("initial xDS derivation did not finish")
			return
		}
		select {
		case <-reconciler.ready:
			synchronized <- nil
		case <-ctx.Done():
			synchronized <- ctx.Err()
		}
	}()
	select {
	case err := <-synchronized:
		if err != nil {
			return err
		}
	case err := <-errCh:
		return err
	case <-ctx.Done():
		return nil
	}
	ready.Store(true)
	select {
	case <-ctx.Done():
		return nil
	case err := <-errCh:
		return err
	}
}
