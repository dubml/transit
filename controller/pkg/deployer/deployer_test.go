package controlplane

import (
	"reflect"
	"slices"
	"testing"

	apps "k8s.io/api/apps/v1"
	core "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	gw "sigs.k8s.io/gateway-api/apis/v1"
)

func TestGatewayPortsRemainStableWhileCertificateUnavailable(t *testing.T) {
	certificate, _ := testServingCertificate(t)
	options := DeploymentOptions{Image: "transit:test", XDSAddress: "https://controller.example:18000", RootCA: string(certificate), Replicas: 1, ServiceType: core.ServiceTypeNodePort}
	gateway := gw.Gateway{ObjectMeta: metav1.ObjectMeta{Name: "edge", Namespace: "team", UID: "gateway"}, Spec: gw.GatewaySpec{Listeners: []gw.Listener{
		{Name: "http", Protocol: gw.HTTPProtocolType, Port: 26080},
		{Name: "https", Protocol: gw.HTTPSProtocolType, Port: 26443},
	}}}
	complete := xdsTestOutput(26080)
	complete.Resources[ListenerType]["https"] = xdsTestOutput(26443).Resources[ListenerType]["http"]
	want, err := DesiredGatewayResources(gateway, complete, options)
	if err != nil {
		t.Fatal(err)
	}
	for _, output := range []GatewayOutput{xdsTestOutput(26080), {}} {
		actual, err := DesiredGatewayResources(gateway, output, options)
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(actual, want) {
			t.Fatal("temporary xDS withdrawal changed declared Service or Deployment ports")
		}
	}
	gateway.Spec.Listeners = gateway.Spec.Listeners[:1]
	resources, err := DesiredGatewayResources(gateway, xdsTestOutput(26080), options)
	if err != nil {
		t.Fatal(err)
	}
	for _, resource := range resources {
		if resource.GVR.Resource != "services" {
			continue
		}
		var service core.Service
		if err := runtime.DefaultUnstructuredConverter.FromUnstructured(resource.Object.Object, &service); err != nil {
			t.Fatal(err)
		}
		if len(service.Spec.Ports) != 1 || service.Spec.Ports[0].Port != 26080 {
			t.Fatal("explicitly removed Gateway listener remained exposed")
		}
	}
	gateway.Spec.Listeners[0].Port = 26021
	if _, err := DesiredGatewayResources(gateway, GatewayOutput{}, options); err == nil {
		t.Fatal("declared management port was accepted without xDS resources")
	}
}

func TestGatewayPortsAndManagementIsolation(t *testing.T) {
	certificate, _ := testServingCertificate(t)
	options := DeploymentOptions{Image: "transit:test", XDSAddress: "https://controller.example:18000", RootCA: string(certificate), Replicas: 1}
	gateway := gw.Gateway{ObjectMeta: metav1.ObjectMeta{Name: "edge", Namespace: "team", UID: "gateway"}}
	output := xdsTestOutput(26080)
	output.Resources[ListenerType]["https"] = xdsTestOutput(26443).Resources[ListenerType]["http"]
	resources, err := DesiredGatewayResources(gateway, output, options)
	if err != nil {
		t.Fatal(err)
	}
	foundService, foundDeployment := false, false
	for _, resource := range resources {
		switch resource.GVR.Resource {
		case "services":
			foundService = true
			var service core.Service
			if err := runtime.DefaultUnstructuredConverter.FromUnstructured(resource.Object.Object, &service); err != nil {
				t.Fatal(err)
			}
			if len(service.Spec.Ports) != 2 {
				t.Fatalf("unexpected Service ports: %v", service.Spec.Ports)
			}
			for index, port := range []int32{26080, 26443} {
				if service.Spec.Ports[index].Port != port || service.Spec.Ports[index].TargetPort.IntVal != port {
					t.Fatalf("API port not exposed: %v", service.Spec.Ports)
				}
			}
		case "deployments":
			foundDeployment = true
			var deployment apps.Deployment
			if err := runtime.DefaultUnstructuredConverter.FromUnstructured(resource.Object.Object, &deployment); err != nil {
				t.Fatal(err)
			}
			container := deployment.Spec.Template.Spec.Containers[0]
			if !slices.Contains(container.Args, "--ui-addr=0.0.0.0:26021") {
				t.Fatalf("wrong management argument: %v", container.Args)
			}
			if !slices.ContainsFunc(container.Ports, func(port core.ContainerPort) bool { return port.Name == "management" && port.ContainerPort == 26021 }) {
				t.Fatal("management port missing")
			}
			for _, probe := range []*core.Probe{container.ReadinessProbe, container.LivenessProbe, container.StartupProbe} {
				if probe.HTTPGet.Port.StrVal != "management" {
					t.Fatal("probe does not use management port")
				}
			}
		}
	}
	if !foundService || !foundDeployment {
		t.Fatal("missing data plane resources")
	}
	if _, err := DesiredGatewayResources(gateway, xdsTestOutput(26021), options); err == nil {
		t.Fatal("management port was accepted as an API listener")
	}
}
