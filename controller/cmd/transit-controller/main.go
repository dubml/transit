package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"math"
	"os"
	"os/signal"
	"syscall"

	"github.com/dubml/transit/controller/internal/controlplane"
	core "k8s.io/api/core/v1"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
)

func run(ctx context.Context, args []string) error {
	flags := flag.NewFlagSet("transit-controller", flag.ContinueOnError)
	var kubeconfig string
	var serviceType string
	var replicas int
	options := controlplane.ServerOptions{}
	flags.StringVar(&kubeconfig, "kubeconfig", "", "Explicit kubeconfig for controller development; defaults to in-cluster credentials")
	flags.StringVar(&options.XDSAddress, "xds-address", ":18000", "TLS xDS listen address")
	flags.StringVar(&options.HealthAddress, "health-address", ":18001", "Health and metrics listen address")
	flags.StringVar(&options.CertificateFile, "tls-cert", "", "Mounted xDS certificate file")
	flags.StringVar(&options.PrivateKeyFile, "tls-key", "", "Mounted xDS private key file")
	flags.StringVar(&options.ClusterDomain, "cluster-domain", "cluster.local", "Kubernetes service DNS domain")
	flags.StringVar(&options.RootCAFile, "xds-root-ca", "", "Mounted root CA distributed to data plane Pods")
	flags.StringVar(&options.Deployment.XDSAddress, "xds-public-address", "https://transit-control-plane.transit-system.svc:18000", "TLS xDS address reachable from data plane Pods")
	flags.StringVar(&options.Deployment.Image, "proxy-image", "", "Transit image used for managed Gateway Deployments")
	flags.IntVar(&replicas, "proxy-replicas", 1, "Replicas per managed Gateway")
	flags.StringVar(&serviceType, "service-type", "LoadBalancer", "Managed Service type: LoadBalancer, ClusterIP or NodePort")
	if err := flags.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return nil
		}
		return err
	}
	if flags.NArg() != 0 {
		return fmt.Errorf("unexpected positional arguments")
	}
	if replicas < 1 || replicas > math.MaxInt32 {
		return fmt.Errorf("proxy-replicas must be between 1 and %d", math.MaxInt32)
	}
	options.Deployment.Replicas = int32(replicas)
	options.Deployment.ServiceType = core.ServiceType(serviceType)
	var err error
	if kubeconfig != "" {
		options.Kubernetes, err = clientcmd.BuildConfigFromFlags("", kubeconfig)
	} else {
		options.Kubernetes, err = rest.InClusterConfig()
	}
	if err != nil {
		return fmt.Errorf("configure Kubernetes access: %w", err)
	}
	return controlplane.Run(ctx, options)
}

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	if err := run(ctx, os.Args[1:]); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
