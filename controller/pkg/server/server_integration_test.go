//go:build integration

package controlplane

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"io"
	"math/big"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	xdscore "github.com/dubml/xds-api/core/v1"
	listener "github.com/dubml/xds-api/listener/v1"
	discovery "github.com/dubml/xds-api/service/discovery/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/structpb"
	authentication "k8s.io/api/authentication/v1"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	apiMeta "k8s.io/apimachinery/pkg/api/meta"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime/schema"
	kubeyaml "k8s.io/apimachinery/pkg/util/yaml"
	"k8s.io/client-go/dynamic"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/util/retry"
	"sigs.k8s.io/controller-runtime/pkg/envtest"
	gw "sigs.k8s.io/gateway-api/apis/v1"
	gwclient "sigs.k8s.io/gateway-api/pkg/client/clientset/versioned"
)

func waitFor(t *testing.T, description string, check func() bool) {
	t.Helper()
	deadline := time.Now().Add(30 * time.Second)
	for time.Now().Before(deadline) {
		if check() {
			return
		}
		time.Sleep(25 * time.Millisecond)
	}
	t.Fatal("timed out waiting for " + description)
}

func testListenAddress(t *testing.T) string {
	t.Helper()
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	address := listener.Addr().String()
	if err := listener.Close(); err != nil {
		t.Fatal(err)
	}
	return address
}

func testTLSFiles(t *testing.T) (string, string, string, string) {
	t.Helper()
	public, private, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	certificate := &x509.Certificate{SerialNumber: big.NewInt(1), Subject: pkix.Name{CommonName: "Transit integration test"}, NotBefore: time.Now().Add(-time.Minute), NotAfter: time.Now().Add(time.Hour), IsCA: true, BasicConstraintsValid: true, KeyUsage: x509.KeyUsageCertSign | x509.KeyUsageDigitalSignature, ExtKeyUsage: []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth}, IPAddresses: []net.IP{net.ParseIP("127.0.0.1")}}
	der, err := x509.CreateCertificate(rand.Reader, certificate, certificate, public, private)
	if err != nil {
		t.Fatal(err)
	}
	key, err := x509.MarshalPKCS8PrivateKey(private)
	if err != nil {
		t.Fatal(err)
	}
	ca := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: der}))
	servingPublic, servingPrivate, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	serving := &x509.Certificate{SerialNumber: big.NewInt(2), Subject: pkix.Name{CommonName: "Transit xDS"}, NotBefore: time.Now().Add(-time.Minute), NotAfter: time.Now().Add(time.Hour), KeyUsage: x509.KeyUsageDigitalSignature, ExtKeyUsage: []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth}, IPAddresses: []net.IP{net.ParseIP("127.0.0.1")}}
	servingDER, err := x509.CreateCertificate(rand.Reader, serving, certificate, servingPublic, private)
	if err != nil {
		t.Fatal(err)
	}
	key, err = x509.MarshalPKCS8PrivateKey(servingPrivate)
	if err != nil {
		t.Fatal(err)
	}
	directory := t.TempDir()
	certFile, keyFile, caFile := filepath.Join(directory, "tls.crt"), filepath.Join(directory, "tls.key"), filepath.Join(directory, "ca.crt")
	if err := os.WriteFile(caFile, []byte(ca), 0600); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(certFile, pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: servingDER}), 0600); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(keyFile, pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: key}), 0600); err != nil {
		t.Fatal(err)
	}
	return certFile, keyFile, caFile, ca
}

func TestControllerAPIServerLifecycle(t *testing.T) {
	module, err := exec.Command("go", "list", "-m", "-f", "{{.Dir}}", "sigs.k8s.io/gateway-api").Output()
	if err != nil {
		t.Fatal(err)
	}
	crdDirectory := filepath.Join(strings.TrimSpace(string(module)), "config/crd/standard")
	paths := []string{"../../install/crds"}
	for _, resource := range []string{"gatewayclasses", "gateways", "httproutes", "referencegrants"} {
		paths = append(paths, filepath.Join(crdDirectory, "gateway.networking.k8s.io_"+resource+".yaml"))
	}
	assets := os.Getenv("KUBEBUILDER_ASSETS")
	download := assets == ""
	if assets == "" {
		assets = filepath.Join(os.TempDir(), "transit-mode-envtest")
	}
	environment := &envtest.Environment{CRDDirectoryPaths: paths, ErrorIfCRDPathMissing: true, BinaryAssetsDirectory: assets, DownloadBinaryAssets: download, DownloadBinaryAssetsVersion: "1.36.0", ControlPlaneStartTimeout: time.Minute}
	config, err := environment.Start()
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := environment.Stop(); err != nil {
			t.Error(err)
		}
	})
	ctx, cancel := context.WithCancel(t.Context())
	defer cancel()
	kube, err := kubernetes.NewForConfig(config)
	if err != nil {
		t.Fatal(err)
	}
	gateways, err := gwclient.NewForConfig(config)
	if err != nil {
		t.Fatal(err)
	}
	dynamicClient, err := dynamic.NewForConfig(config)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := kube.CoreV1().Namespaces().Create(ctx, &core.Namespace{ObjectMeta: metav1.ObjectMeta{Name: "team"}}, metav1.CreateOptions{}); err != nil {
		t.Fatal(err)
	}
	if _, err := kube.CoreV1().Namespaces().Create(ctx, &core.Namespace{ObjectMeta: metav1.ObjectMeta{Name: "transit-system"}}, metav1.CreateOptions{}); err != nil {
		t.Fatal(err)
	}
	rbac, err := os.ReadFile("../../install/rbac.yaml")
	if err != nil {
		t.Fatal(err)
	}
	decoder := kubeyaml.NewYAMLOrJSONDecoder(bytes.NewReader(rbac), 4096)
	for {
		var object unstructured.Unstructured
		if err := decoder.Decode(&object); err == io.EOF {
			break
		} else if err != nil {
			t.Fatal(err)
		}
		resources := map[string]string{"ServiceAccount": "serviceaccounts", "ClusterRole": "clusterroles", "ClusterRoleBinding": "clusterrolebindings"}
		resource := resources[object.GetKind()]
		if resource == "" {
			t.Fatalf("unexpected RBAC manifest kind %s", object.GetKind())
		}
		gv := object.GroupVersionKind().GroupVersion()
		api := dynamicClient.Resource(schema.GroupVersionResource{Group: gv.Group, Version: gv.Version, Resource: resource})
		if object.GetNamespace() == "" {
			_, err = api.Create(ctx, &object, metav1.CreateOptions{})
		} else {
			_, err = api.Namespace(object.GetNamespace()).Create(ctx, &object, metav1.CreateOptions{})
		}
		if err != nil {
			t.Fatal(err)
		}
	}
	controllerToken, err := kube.CoreV1().ServiceAccounts("transit-system").CreateToken(ctx, "transit-controller", &authentication.TokenRequest{}, metav1.CreateOptions{})
	if err != nil {
		t.Fatal(err)
	}
	controllerConfig := rest.CopyConfig(config)
	controllerConfig.CertData, controllerConfig.KeyData = nil, nil
	controllerConfig.CertFile, controllerConfig.KeyFile = "", ""
	controllerConfig.BearerToken = controllerToken.Status.Token
	if _, err := gateways.GatewayV1().GatewayClasses().Create(ctx, &gw.GatewayClass{ObjectMeta: metav1.ObjectMeta{Name: "transit"}, Spec: gw.GatewayClassSpec{ControllerName: gw.GatewayController(ControllerName)}}, metav1.CreateOptions{}); err != nil {
		t.Fatal(err)
	}
	gateway, err := gateways.GatewayV1().Gateways("team").Create(ctx, &gw.Gateway{ObjectMeta: metav1.ObjectMeta{Name: "edge"}, Spec: gw.GatewaySpec{GatewayClassName: "transit", Listeners: []gw.Listener{{Name: "http", Port: 8080, Protocol: gw.HTTPProtocolType}}}}, metav1.CreateOptions{})
	if err != nil {
		t.Fatal(err)
	}
	certFile, keyFile, caFile, ca := testTLSFiles(t)
	xdsAddress, healthAddress := testListenAddress(t), testListenAddress(t)
	stopped := make(chan error, 1)
	go func() {
		stopped <- Run(ctx, ServerOptions{Kubernetes: controllerConfig, XDSAddress: xdsAddress, HealthAddress: healthAddress, CertificateFile: certFile, PrivateKeyFile: keyFile, RootCAFile: caFile, Deployment: DeploymentOptions{Image: "transit:test", XDSAddress: "https://" + xdsAddress, Replicas: 1, ServiceType: core.ServiceTypeNodePort}})
	}()
	t.Cleanup(func() {
		cancel()
		select {
		case err := <-stopped:
			if err != nil {
				t.Error(err)
			}
		case <-time.After(10 * time.Second):
			t.Error("controller did not stop")
		}
	})
	healthClient := &http.Client{Timeout: time.Second}
	waitFor(t, "controller readiness", func() bool {
		response, err := healthClient.Get("http://" + healthAddress + "/readyz")
		if err != nil {
			return false
		}
		defer response.Body.Close()
		return response.StatusCode == http.StatusOK
	})
	name := DataPlaneName("edge")
	waitFor(t, "owned data plane Deployment", func() bool {
		deployment, err := kube.AppsV1().Deployments("team").Get(ctx, name, metav1.GetOptions{})
		return err == nil && len(deployment.OwnerReferences) == 1 && deployment.OwnerReferences[0].UID == gateway.UID
	})
	deployment, err := kube.AppsV1().Deployments("team").Get(ctx, name, metav1.GetOptions{})
	if err != nil {
		t.Fatal(err)
	}
	if deployment.Spec.Template.Spec.ServiceAccountName != name {
		t.Fatal("data plane has incorrect identity")
	}
	token, err := kube.CoreV1().ServiceAccounts("team").CreateToken(ctx, name, &authentication.TokenRequest{Spec: authentication.TokenRequestSpec{Audiences: []string{XDSAudience}}}, metav1.CreateOptions{})
	if err != nil {
		t.Fatal(err)
	}
	roots := x509.NewCertPool()
	roots.AppendCertsFromPEM([]byte(ca))
	connection, err := grpc.NewClient(xdsAddress, grpc.WithTransportCredentials(credentials.NewTLS(&tls.Config{MinVersion: tls.VersionTLS12, RootCAs: roots})))
	if err != nil {
		t.Fatal(err)
	}
	defer connection.Close()
	authenticated, stopStream := context.WithTimeout(metadata.NewOutgoingContext(ctx, metadata.Pairs("authorization", "Bearer "+token.Status.Token)), time.Minute)
	defer stopStream()
	stream, err := discovery.NewAggregatedDiscoveryServiceClient(connection).DeltaAggregatedResources(authenticated)
	if err != nil {
		t.Fatal(err)
	}
	node := &xdscore.Node{Id: "integration-proxy", Metadata: &structpb.Struct{Fields: map[string]*structpb.Value{"TRANSIT_GATEWAY": structpb.NewStringValue("team/edge")}}}
	for _, resourceType := range []string{ListenerType, RouteType} {
		if err := stream.Send(&discovery.DeltaDiscoveryRequest{Node: node, TypeUrl: resourceType, ResourceNamesSubscribe: []string{"*"}}); err != nil {
			t.Fatal(err)
		}
		response, err := stream.Recv()
		if err != nil {
			t.Fatal(err)
		}
		if response.TypeUrl != resourceType || len(response.Resources) != 1 {
			t.Fatalf("unexpected xDS response: %s", response)
		}
		if err := stream.Send(&discovery.DeltaDiscoveryRequest{TypeUrl: resourceType, ResponseNonce: response.Nonce}); err != nil {
			t.Fatal(err)
		}
	}
	waitFor(t, "Gateway pending before deployment readiness", func() bool {
		value, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
		return err == nil && apiMeta.IsStatusConditionFalse(value.Status.Conditions, "Programmed")
	})
	markReady := func() {
		if err := retry.RetryOnConflict(retry.DefaultRetry, func() error {
			deployment, err := kube.AppsV1().Deployments("team").Get(ctx, name, metav1.GetOptions{})
			if err != nil {
				return err
			}
			deployment.Status.ObservedGeneration = deployment.Generation
			deployment.Status.Replicas, deployment.Status.UpdatedReplicas, deployment.Status.AvailableReplicas, deployment.Status.ReadyReplicas = 1, 1, 1, 1
			_, err = kube.AppsV1().Deployments("team").UpdateStatus(ctx, deployment, metav1.UpdateOptions{})
			return err
		}); err != nil {
			t.Fatal(err)
		}
	}
	markReady()
	waitFor(t, "Programmed after ACK and deployment readiness", func() bool {
		value, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
		return err == nil && apiMeta.IsStatusConditionTrue(value.Status.Conditions, "Programmed") && len(value.Status.Addresses) == 1
	})
	t.Run("reject a different Gateway identity", func(t *testing.T) {
		other, err := discovery.NewAggregatedDiscoveryServiceClient(connection).DeltaAggregatedResources(authenticated)
		if err != nil {
			t.Fatal(err)
		}
		if err := other.Send(&discovery.DeltaDiscoveryRequest{Node: &xdscore.Node{Id: "other", Metadata: &structpb.Struct{Fields: map[string]*structpb.Value{"TRANSIT_GATEWAY": structpb.NewStringValue("team/other")}}}, TypeUrl: ListenerType}); err != nil {
			t.Fatal(err)
		}
		_, err = other.Recv()
		if status.Code(err) != codes.PermissionDenied {
			t.Fatalf("wrong identity returned %v", err)
		}
	})
	t.Run("stable deployment does not churn", func(t *testing.T) {
		before, err := kube.AppsV1().Deployments("team").Get(ctx, name, metav1.GetOptions{})
		if err != nil {
			t.Fatal(err)
		}
		if err := retry.RetryOnConflict(retry.DefaultRetry, func() error {
			value, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
			if err != nil {
				return err
			}
			value.Annotations = map[string]string{"test.transit.dev/reconcile": "again"}
			_, err = gateways.GatewayV1().Gateways("team").Update(ctx, value, metav1.UpdateOptions{})
			return err
		}); err != nil {
			t.Fatal(err)
		}
		deadline := time.Now().Add(time.Second)
		for time.Now().Before(deadline) {
			current, err := kube.AppsV1().Deployments("team").Get(ctx, name, metav1.GetOptions{})
			if err != nil {
				t.Fatal(err)
			}
			if current.ResourceVersion != before.ResourceVersion {
				t.Fatal("identical desired deployment was rewritten")
			}
			time.Sleep(50 * time.Millisecond)
		}
	})
	t.Run("changed listener needs fresh acknowledgement", func(t *testing.T) {
		defer func() {
			if !t.Failed() {
				return
			}
			if current, err := kube.AppsV1().Deployments("team").Get(ctx, name, metav1.GetOptions{}); err == nil {
				t.Logf("actual deployment ports: %+v", current.Spec.Template.Spec.Containers[0].Ports)
			}
			if current, err := kube.CoreV1().Services("team").Get(ctx, name, metav1.GetOptions{}); err == nil {
				t.Logf("actual service ports: %+v", current.Spec.Ports)
			}
		}()
		value, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
		if err != nil {
			t.Fatal(err)
		}
		value.Spec.Listeners[0].Port = 9090
		value, err = gateways.GatewayV1().Gateways("team").Update(ctx, value, metav1.UpdateOptions{})
		if err != nil {
			t.Fatal(err)
		}
		response, err := stream.Recv()
		if err != nil {
			t.Fatal(err)
		}
		published := &listener.Listener{}
		if response.TypeUrl != ListenerType || len(response.Resources) != 1 {
			t.Fatalf("unexpected update: %s", response)
		}
		if err := response.Resources[0].Resource.UnmarshalTo(published); err != nil {
			t.Fatal(err)
		}
		if published.GetAddress().GetSocketAddress().GetPortValue() != 9090 {
			t.Fatal("listener port update was lost")
		}
		waitFor(t, "current Gateway generation pending", func() bool {
			current, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
			if err != nil {
				return false
			}
			condition := apiMeta.FindStatusCondition(current.Status.Conditions, "Programmed")
			return condition != nil && condition.ObservedGeneration == value.Generation && condition.Status == metav1.ConditionFalse
		})
		waitFor(t, "deployment listener update", func() bool {
			current, err := kube.AppsV1().Deployments("team").Get(ctx, name, metav1.GetOptions{})
			return err == nil && current.Spec.Template.Spec.Containers[0].Ports[0].ContainerPort == 9090
		})
		markReady()
		if err := stream.Send(&discovery.DeltaDiscoveryRequest{TypeUrl: ListenerType, ResponseNonce: response.Nonce}); err != nil {
			t.Fatal(err)
		}
		waitFor(t, "updated Gateway programmed", func() bool {
			current, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
			return err == nil && apiMeta.IsStatusConditionTrue(current.Status.Conditions, "Programmed")
		})
	})
	t.Run("adding a listener preserves existing NodePort allocation", func(t *testing.T) {
		before, err := kube.CoreV1().Services("team").Get(ctx, name, metav1.GetOptions{})
		if err != nil {
			t.Fatal(err)
		}
		nodePort := before.Spec.Ports[0].NodePort
		if nodePort == 0 {
			t.Fatal("API Server did not allocate a NodePort")
		}
		if err := retry.RetryOnConflict(retry.DefaultRetry, func() error {
			value, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
			if err != nil {
				return err
			}
			value.Spec.Listeners = append(value.Spec.Listeners, gw.Listener{Name: "additional", Port: 9091, Protocol: gw.HTTPProtocolType})
			_, err = gateways.GatewayV1().Gateways("team").Update(ctx, value, metav1.UpdateOptions{})
			return err
		}); err != nil {
			t.Fatal(err)
		}
		waitFor(t, "both listener ports with stable NodePort", func() bool {
			value, err := kube.CoreV1().Services("team").Get(ctx, name, metav1.GetOptions{})
			if err != nil || len(value.Spec.Ports) != 2 {
				return false
			}
			return value.Spec.Ports[0].Port == 9090 && value.Spec.Ports[0].NodePort == nodePort && value.Spec.Ports[1].Port == 9091 && value.Spec.Ports[1].NodePort != 0
		})
	})
	t.Run("do not adopt user resources", func(t *testing.T) {
		conflictName := DataPlaneName("conflict")
		if _, err := kube.CoreV1().ConfigMaps("team").Create(ctx, &core.ConfigMap{ObjectMeta: metav1.ObjectMeta{Name: conflictName}, Data: map[string]string{"owner": "user"}}, metav1.CreateOptions{}); err != nil {
			t.Fatal(err)
		}
		if _, err := gateways.GatewayV1().Gateways("team").Create(ctx, &gw.Gateway{ObjectMeta: metav1.ObjectMeta{Name: "conflict"}, Spec: gateway.Spec}, metav1.CreateOptions{}); err != nil {
			t.Fatal(err)
		}
		waitFor(t, "ownership conflict status", func() bool {
			value, err := gateways.GatewayV1().Gateways("team").Get(ctx, "conflict", metav1.GetOptions{})
			if err != nil {
				return false
			}
			condition := apiMeta.FindStatusCondition(value.Status.Conditions, "Programmed")
			return condition != nil && condition.Status == metav1.ConditionFalse && strings.Contains(condition.Message, "refusing to adopt")
		})
		value, err := kube.CoreV1().ConfigMaps("team").Get(ctx, conflictName, metav1.GetOptions{})
		if err != nil {
			t.Fatal(err)
		}
		if value.Data["owner"] != "user" || len(value.OwnerReferences) != 0 {
			t.Fatal("user ConfigMap was changed")
		}
	})
	t.Run("invalid listeners withdraw resources and status", func(t *testing.T) {
		value, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
		if err != nil {
			t.Fatal(err)
		}
		for i := range value.Spec.Listeners {
			value.Spec.Listeners[i].Protocol = gw.TCPProtocolType
		}
		if _, err = gateways.GatewayV1().Gateways("team").Update(ctx, value, metav1.UpdateOptions{}); err != nil {
			t.Fatal(err)
		}
		waitFor(t, "invalid Gateway status and removed data plane", func() bool {
			value, err := gateways.GatewayV1().Gateways("team").Get(ctx, "edge", metav1.GetOptions{})
			if err != nil || !apiMeta.IsStatusConditionFalse(value.Status.Conditions, "Accepted") || !apiMeta.IsStatusConditionFalse(value.Status.Conditions, "Programmed") || len(value.Status.Addresses) != 0 {
				return false
			}
			for _, resource := range managedResourceTypes {
				_, err := dynamicClient.Resource(resource).Namespace("team").Get(ctx, name, metav1.GetOptions{})
				if !apierrors.IsNotFound(err) {
					return false
				}
			}
			return true
		})
	})
	t.Run("route statuses preserve all parents and foreign controllers", func(t *testing.T) {
		for _, name := range []string{"first", "second"} {
			if _, err := gateways.GatewayV1().Gateways("team").Create(ctx, &gw.Gateway{ObjectMeta: metav1.ObjectMeta{Name: name}, Spec: gateway.Spec}, metav1.CreateOptions{}); err != nil {
				t.Fatal(err)
			}
		}
		if _, err := kube.CoreV1().Services("team").Create(ctx, &core.Service{ObjectMeta: metav1.ObjectMeta{Name: "upstream"}, Spec: core.ServiceSpec{Ports: []core.ServicePort{{Port: 9000}}}}, metav1.CreateOptions{}); err != nil {
			t.Fatal(err)
		}
		port := gw.PortNumber(9000)
		_, err := gateways.GatewayV1().HTTPRoutes("team").Create(ctx, &gw.HTTPRoute{ObjectMeta: metav1.ObjectMeta{Name: "shared"}, Spec: gw.HTTPRouteSpec{CommonRouteSpec: gw.CommonRouteSpec{ParentRefs: []gw.ParentReference{{Name: "first"}, {Name: "second"}}}, Rules: []gw.HTTPRouteRule{{BackendRefs: []gw.HTTPBackendRef{{BackendRef: gw.BackendRef{BackendObjectReference: gw.BackendObjectReference{Name: "upstream", Port: &port}}}}}}}}, metav1.CreateOptions{})
		if err != nil {
			t.Fatal(err)
		}
		waitFor(t, "both parent statuses", func() bool {
			value, err := gateways.GatewayV1().HTTPRoutes("team").Get(ctx, "shared", metav1.GetOptions{})
			return err == nil && len(value.Status.Parents) == 2 && apiMeta.IsStatusConditionTrue(value.Status.Parents[0].Conditions, "Accepted") && apiMeta.IsStatusConditionTrue(value.Status.Parents[1].Conditions, "Accepted")
		})
		foreign := gw.GatewayController("other.example/controller")
		if err := retry.RetryOnConflict(retry.DefaultRetry, func() error {
			value, err := gateways.GatewayV1().HTTPRoutes("team").Get(ctx, "shared", metav1.GetOptions{})
			if err != nil {
				return err
			}
			value.Status.Parents = append(value.Status.Parents, gw.RouteParentStatus{ParentRef: gw.ParentReference{Name: "first"}, ControllerName: foreign, Conditions: []metav1.Condition{{Type: "Accepted", Status: metav1.ConditionTrue, Reason: "Accepted", Message: "Foreign controller status", ObservedGeneration: value.Generation, LastTransitionTime: metav1.Now()}}})
			_, err = gateways.GatewayV1().HTTPRoutes("team").UpdateStatus(ctx, value, metav1.UpdateOptions{})
			return err
		}); err != nil {
			t.Fatal(err)
		}
		if err := retry.RetryOnConflict(retry.DefaultRetry, func() error {
			value, err := gateways.GatewayV1().HTTPRoutes("team").Get(ctx, "shared", metav1.GetOptions{})
			if err != nil {
				return err
			}
			value.Spec.ParentRefs = []gw.ParentReference{{Name: "second"}}
			_, err = gateways.GatewayV1().HTTPRoutes("team").Update(ctx, value, metav1.UpdateOptions{})
			return err
		}); err != nil {
			t.Fatal(err)
		}
		waitFor(t, "removed parent pruned while foreign status survives", func() bool {
			value, err := gateways.GatewayV1().HTTPRoutes("team").Get(ctx, "shared", metav1.GetOptions{})
			if err != nil || len(value.Status.Parents) != 2 {
				return false
			}
			ours, theirs := false, false
			for _, parent := range value.Status.Parents {
				if parent.ControllerName == foreign {
					theirs = true
				}
				if string(parent.ControllerName) == ControllerName && parent.ParentRef.Name == "second" && apiMeta.IsStatusConditionTrue(parent.Conditions, "Accepted") {
					ours = true
				}
			}
			return ours && theirs
		})
	})
	// Ensure the bundled CRD is usable by the same dynamic path as the compiler.
	serviceType, _ := resourceType("TransitService")
	_, err = dynamicClient.Resource(serviceType.GVR).Namespace("team").Create(ctx, &unstructured.Unstructured{Object: map[string]any{"apiVersion": TransitGroup + "/" + TransitVersion, "kind": "TransitService", "metadata": map[string]any{"name": "chat"}, "spec": map[string]any{"ai": map[string]any{"provider": map[string]any{"openai": map[string]any{}}}}}}, metav1.CreateOptions{})
	if err != nil {
		t.Fatal(err)
	}
	t.Run("real Rust proxy with HTTP HTTPS and TransitService traffic", func(t *testing.T) {
		testRustProxyTraffic(t, ctx, kube, gateways, dynamicClient, xdsAddress, caFile)
	})
}
