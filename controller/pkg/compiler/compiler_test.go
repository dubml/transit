package controlplane

import (
	"encoding/json"
	"fmt"
	"testing"
	"time"

	route "github.com/dubml/xds-api/route/v1"
	"istio.io/istio/pkg/kube/krt"
	apiMeta "k8s.io/apimachinery/pkg/api/meta"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
)

func TestGatewayPartialListenersAndClassParameters(t *testing.T) {
	inputs, outputs := testGateway(t, `{"ai":{"provider":{"openai":{}}}}`)
	inputJSON(t, inputs, "Gateway", "team", "edge", `{"gatewayClassName":"transit","listeners":[{"name":"http","port":8080,"protocol":"HTTP"},{"name":"tcp","port":9090,"protocol":"TCP"}]}`)
	output := awaitGateway(t, outputs, func(value *GatewayOutput) bool {
		for _, status := range value.Statuses {
			if status.Kind == "Gateway" && len(status.Listeners) == 2 {
				return true
			}
		}
		return false
	})
	for _, status := range output.Statuses {
		if status.Kind == "Gateway" && !apiMeta.IsStatusConditionTrue(status.Conditions, "Accepted") {
			t.Fatal("a valid HTTP listener must remain accepted")
		}
	}
	inputJSON(t, inputs, "GatewayClass", "", "transit", `{"controllerName":"transit.dev/gateway-controller","parametersRef":{"group":"example.com","kind":"Settings","name":"unsupported"}}`)
	output = awaitGateway(t, outputs, func(value *GatewayOutput) bool { return len(value.Resources[ListenerType]) == 0 })
	if len(output.Statuses) == 0 || !apiMeta.IsStatusConditionFalse(output.Statuses[0].Conditions, "Accepted") {
		t.Fatal("unsupported class parameters must reject the Gateway")
	}
}

func inputJSON(t *testing.T, inputs Inputs, kind, namespace, name, spec string) {
	t.Helper()
	obj := map[string]any{}
	if err := json.Unmarshal([]byte(fmt.Sprintf(`{"metadata":{"name":%q,"namespace":%q,"generation":1},"spec":%s}`, name, namespace, spec)), &obj); err != nil {
		t.Fatal(err)
	}
	inputs.Objects.ConditionalUpdateObject(NewResource(kind, &unstructured.Unstructured{Object: obj}))
}

func awaitGateway(t *testing.T, outputs krt.Collection[GatewayOutput], predicate func(*GatewayOutput) bool) *GatewayOutput {
	t.Helper()
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		if output := outputs.GetKey("team/edge"); output != nil && predicate(output) {
			return output
		}
		time.Sleep(5 * time.Millisecond)
	}
	t.Fatalf("Gateway output did not converge: %#v", outputs.GetKey("team/edge"))
	return nil
}

func testGateway(t *testing.T, service string) (Inputs, krt.Collection[GatewayOutput]) {
	t.Helper()
	inputs := NewInputs(nil, t.Context().Done())
	inputJSON(t, inputs, "GatewayClass", "", "transit", `{"controllerName":"transit.dev/gateway-controller"}`)
	inputJSON(t, inputs, "Gateway", "team", "edge", `{"gatewayClassName":"transit","listeners":[{"name":"http","port":8080,"protocol":"HTTP"}]}`)
	inputJSON(t, inputs, "Service", "team", "upstream", `{"clusterIP":"127.0.0.1","ports":[{"port":9000,"protocol":"TCP"}]}`)
	inputJSON(t, inputs, "TransitService", "team", "backend", service)
	inputJSON(t, inputs, "HTTPRoute", "team", "route", `{"parentRefs":[{"name":"edge"}],"rules":[{"backendRefs":[{"group":"networking.dubbo.apache.org","kind":"TransitService","name":"backend"}]}]}`)
	return inputs, NewOutputs(inputs, ControllerName, "cluster.local", t.Context().Done())
}

func TestCanonicalTransitServiceVariantsCompile(t *testing.T) {
	for _, tc := range []struct {
		name, spec string
		protocol   route.AgentProtocol
		endpoint   string
	}{
		{"LLM default endpoint", `{"ai":{"provider":{"openai":{}}}}`, route.AgentProtocol_LLM, "https://api.openai.com/v1"},
		{"MCP static backend", `{"mcp":{"targets":[{"name":"tools","static":{"backendRef":{"name":"upstream"},"port":9000,"path":"/mcp"},"tools":["read"]}]}}`, route.AgentProtocol_MCP, "http://127.0.0.1:9000/mcp"},
		{"A2A service backend", `{"a2a":{"backendRef":{"name":"upstream"},"port":9000,"path":"/agent"}}`, route.AgentProtocol_A2A, "http://127.0.0.1:9000/agent"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			_, outputs := testGateway(t, tc.spec)
			out := awaitGateway(t, outputs, func(out *GatewayOutput) bool { return len(out.Resources[RouteType]) > 0 })
			config := &route.RouteConfiguration{}
			if err := out.Resources[RouteType]["team/edge/http"].Value.UnmarshalTo(config); err != nil {
				t.Fatal(err)
			}
			if len(config.AgentConfig.AgentRoutes) != 1 {
				t.Fatalf("expected routable service, got statuses: %+v", out.Statuses)
			}
			if got := config.AgentConfig.AgentRoutes[0]; got.Protocol != tc.protocol || len(got.ListenerPorts) != 1 || got.ListenerPorts[0] != 8080 {
				t.Fatalf("incorrect route scope: %v", got)
			}
			backend := config.AgentConfig.Backends[0]
			var endpoint string
			switch tc.protocol {
			case route.AgentProtocol_LLM:
				endpoint = backend.GetLlm().Endpoint
			case route.AgentProtocol_MCP:
				endpoint = backend.GetMcp().Endpoint
			case route.AgentProtocol_A2A:
				endpoint = backend.GetA2A().Endpoint
			}
			if endpoint != tc.endpoint {
				t.Fatalf("endpoint %q, expected %q", endpoint, tc.endpoint)
			}
		})
	}
}

func TestKRTReconcilesBackendChangesAndDeletion(t *testing.T) {
	inputs, outputs := testGateway(t, `{"a2a":{"backendRef":{"name":"upstream"},"port":9000}}`)
	initial := awaitGateway(t, outputs, func(out *GatewayOutput) bool { return len(out.Resources[RouteType]) > 0 })
	version := initial.Version
	inputJSON(t, inputs, "Service", "team", "upstream", `{"clusterIP":"127.0.0.2","ports":[{"port":9000,"protocol":"TCP"}]}`)
	updated := awaitGateway(t, outputs, func(out *GatewayOutput) bool { return out.Version != version })
	config := &route.RouteConfiguration{}
	if err := updated.Resources[RouteType]["team/edge/http"].Value.UnmarshalTo(config); err != nil {
		t.Fatal(err)
	}
	if config.AgentConfig.Backends[0].GetA2A().Endpoint != "http://127.0.0.2:9000" {
		t.Fatal("dependency update did not reach xDS")
	}
	version = updated.Version
	inputs.Objects.DeleteObject(resourceKey("Service", "team", "upstream"))
	deleted := awaitGateway(t, outputs, func(out *GatewayOutput) bool { return out.Version != version })
	if err := deleted.Resources[RouteType]["team/edge/http"].Value.UnmarshalTo(config); err != nil {
		t.Fatal(err)
	}
	if len(config.AgentConfig.AgentRoutes) != 0 {
		t.Fatal("deleted backend remains routable")
	}
}
