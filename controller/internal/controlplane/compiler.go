package controlplane

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"maps"
	"slices"
	"strings"

	core "github.com/kdubbo/xds-api/core/v1"
	hcm "github.com/kdubbo/xds-api/extensions/filters/v1/network/http_connection_manager"
	listener "github.com/kdubbo/xds-api/listener/v1"
	route "github.com/kdubbo/xds-api/route/v1"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"istio.io/istio/pkg/kube/krt"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/labels"
	gw "sigs.k8s.io/gateway-api/apis/v1"
	grant "sigs.k8s.io/gateway-api/apis/v1beta1"
)

type StatusUpdate struct {
	Kind string `json:"kind"`
	Namespace string `json:"namespace"`
	Name string `json:"name"`
	Generation int64 `json:"generation"`
	Parent *gw.ParentReference `json:"parent,omitempty"`
	Conditions []metav1.Condition `json:"conditions"`
	Listeners []gw.ListenerStatus `json:"listeners,omitempty"`
}

func condition(kind string, ok bool, reason, message string, generation int64) metav1.Condition {
	state := metav1.ConditionFalse; if ok { state = metav1.ConditionTrue }
	return metav1.Condition{Type: kind, Status: state, Reason: reason, Message: message, ObservedGeneration: generation}
}

type Compiler struct { Inputs Inputs; ControllerName string; ClusterDomain string }

func NewOutputs(inputs Inputs, controllerName, clusterDomain string, stop <-chan struct{}) krt.Collection[GatewayOutput] {
	gateways := krt.NewCollection(inputs.Objects, func(_ krt.HandlerContext, r Resource) *Resource {
		if r.Kind != "Gateway" { return nil }; return &r
	}, krt.WithName("transit/gateways"), krt.WithStop(stop))
	c := Compiler{Inputs: inputs, ControllerName: controllerName, ClusterDomain: clusterDomain}
	return krt.NewCollection(gateways, c.CompileGateway, krt.WithName("transit/xds-outputs"), krt.WithStop(stop))
}

func (c Compiler) CompileGateway(ctx krt.HandlerContext, input Resource) *GatewayOutput {
	var gateway gw.Gateway
	if err := input.Decode(&gateway); err != nil { return nil }
	class := c.Inputs.Get(ctx, "GatewayClass", "", string(gateway.Spec.GatewayClassName))
	if class == nil { return nil }
	var gatewayClass gw.GatewayClass
	if class.Decode(&gatewayClass) != nil || string(gatewayClass.Spec.ControllerName) != c.ControllerName { return nil }
	out := &GatewayOutput{Key: gatewayKey(gateway.Namespace, gateway.Name), Resources: map[string]map[string]EncodedResource{}}
	out.Statuses = append(out.Statuses, StatusUpdate{Kind: "GatewayClass", Name: gatewayClass.Name, Generation: gatewayClass.Generation,
		Conditions: []metav1.Condition{condition("Accepted", true, "Accepted", "GatewayClass is handled by the Transit controller", gatewayClass.Generation)}})
	routes := krt.Fetch(ctx, c.Inputs.Objects, krt.FilterIndex(c.Inputs.RoutesByGateway, out.Key))
	slices.SortFunc(routes, func(a, b Resource) int { return strings.Compare(a.ResourceName(), b.ResourceName()) })
	gatewayStatus := StatusUpdate{Kind: "Gateway", Namespace: gateway.Namespace, Name: gateway.Name, Generation: gateway.Generation}
	accepted := true
	for _, l := range gateway.Spec.Listeners {
		ls := gw.ListenerStatus{Name: l.Name, SupportedKinds: []gw.RouteGroupKind{{Kind: "HTTPRoute"}}}
		if l.Protocol != gw.HTTPProtocolType {
			accepted = false
			ls.Conditions = []metav1.Condition{condition("Accepted", false, "UnsupportedProtocol", "This listener requires a supported HTTP transport", gateway.Generation)}
			gatewayStatus.Listeners = append(gatewayStatus.Listeners, ls)
			continue
		}
		name := out.Key + "/" + string(l.Name)
		routeConfig := &route.RouteConfiguration{Name: name, AgentConfig: &route.AgentConfig{}}
		for _, r := range routes {
			var httpRoute gw.HTTPRoute
			if r.Decode(&httpRoute) != nil { continue }
			parents := matchingParents(httpRoute, gateway, l)
			if len(parents) == 0 { continue }
			hosts := intersectHostnames(l.Hostname, httpRoute.Spec.Hostnames)
			allowed := c.routeNamespaceAllowed(ctx, gateway, l, httpRoute.Namespace)
			if len(hosts) == 0 || !allowed {
				reason := "NotAllowedByListeners"; message := "The listener does not allow this Route namespace"
				if len(hosts) == 0 { reason = "NoMatchingListenerHostname"; message = "Route and listener hostnames do not intersect" }
				for _, parent := range parents { out.Statuses = append(out.Statuses, routeStatus(httpRoute, parent, false, false, reason, message)) }
				continue
			}
			compiled, err := c.compileHTTPRoute(ctx, httpRoute, l, hosts)
			if err != nil {
				for _, parent := range parents { out.Statuses = append(out.Statuses, routeStatus(httpRoute, parent, false, false, "UnsupportedValue", err.Error())) }
				continue
			}
			ls.AttachedRoutes++
			routeConfig.AgentConfig.Providers = append(routeConfig.AgentConfig.Providers, compiled.Providers...)
			routeConfig.AgentConfig.Backends = append(routeConfig.AgentConfig.Backends, compiled.Backends...)
			routeConfig.AgentConfig.AgentRoutes = append(routeConfig.AgentConfig.AgentRoutes, compiled.AgentRoutes...)
			routeConfig.AgentConfig.Policies = append(routeConfig.AgentConfig.Policies, compiled.Policies...)
			for _, parent := range parents { out.Statuses = append(out.Statuses, routeStatus(httpRoute, parent, true, true, "Accepted", "Route is included in the published xDS configuration")) }
		}
		deduplicateAgentConfig(routeConfig.AgentConfig)
		slices.SortStableFunc(routeConfig.AgentConfig.AgentRoutes, compareRoutes)
		manager, _ := anypb.New(&hcm.HttpConnectionManager{StatPrefix: name, RouteSpecifier: &hcm.HttpConnectionManager_Rds{Rds: &hcm.Rds{RouteConfigName: name}}})
		lds := &listener.Listener{Name: name, Address: &core.Address{Address: &core.Address_SocketAddress{SocketAddress: &core.SocketAddress{
			Address: "0.0.0.0", PortSpecifier: &core.SocketAddress_PortValue{PortValue: uint32(l.Port)},
		}}}, ApiListener: &listener.ApiListener{ApiListener: manager}}
		out.add(lds.Name, lds); out.add(routeConfig.Name, routeConfig)
		ls.Conditions = []metav1.Condition{
			condition("Accepted", true, "Accepted", "Listener configuration is valid", gateway.Generation),
			condition("ResolvedRefs", true, "ResolvedRefs", "Listener references are resolved", gateway.Generation),
			condition("Programmed", true, "Programmed", "Listener resources are available through xDS", gateway.Generation),
		}
		gatewayStatus.Listeners = append(gatewayStatus.Listeners, ls)
	}
	reason := "Accepted"; message := "Gateway configuration is available through xDS"
	if !accepted { reason = "ListenersNotValid"; message = "One or more listeners are not supported; inspect listener conditions" }
	gatewayStatus.Conditions = []metav1.Condition{condition("Accepted", accepted, reason, message, gateway.Generation), condition("Programmed", accepted, map[bool]string{true:"Programmed",false:"Invalid"}[accepted], message, gateway.Generation)}
	out.Statuses = append(out.Statuses, gatewayStatus)
	out.finishVersion()
	return out
}

func (out *GatewayOutput) add(name string, message proto.Message) {
	resource := encodeResource(name, message)
	if out.Resources[resource.Value.TypeUrl] == nil { out.Resources[resource.Value.TypeUrl] = map[string]EncodedResource{} }
	out.Resources[resource.Value.TypeUrl][name] = resource
}

func (out *GatewayOutput) finishVersion() {
	hash := sha256.New()
	for _, kind := range slices.Sorted(maps.Keys(out.Resources)) {
		for _, name := range slices.Sorted(maps.Keys(out.Resources[kind])) { hash.Write([]byte(kind + "\x00" + name + "\x00" + out.Resources[kind][name].Version)) }
	}
	statusBytes, _ := json.Marshal(out.Statuses); hash.Write(statusBytes)
	out.Version = hex.EncodeToString(hash.Sum(nil))
}

func routeStatus(r gw.HTTPRoute, parent gw.ParentReference, accepted, resolved bool, reason, message string) StatusUpdate {
	return StatusUpdate{Kind:"HTTPRoute", Namespace:r.Namespace, Name:r.Name, Generation:r.Generation, Parent:&parent,
		Conditions: []metav1.Condition{condition("Accepted", accepted, reason, message, r.Generation), condition("ResolvedRefs", resolved, map[bool]string{true:"ResolvedRefs",false:"BackendNotFound"}[resolved], message, r.Generation)}}
}

func matchingParents(r gw.HTTPRoute, gateway gw.Gateway, listener gw.Listener) []gw.ParentReference {
	var result []gw.ParentReference
	for _, parent := range r.Spec.ParentRefs {
		if valueOr(parent.Group, "gateway.networking.k8s.io") != "gateway.networking.k8s.io" || valueOr(parent.Kind, "Gateway") != "Gateway" { continue }
		if valueOr(parent.Namespace, r.Namespace) != gateway.Namespace || string(parent.Name) != gateway.Name { continue }
		if parent.SectionName != nil && *parent.SectionName != listener.Name { continue }
		if parent.Port != nil && *parent.Port != listener.Port { continue }
		result = append(result, parent)
	}
	return result
}

func (c Compiler) routeNamespaceAllowed(ctx krt.HandlerContext, gateway gw.Gateway, listener gw.Listener, namespace string) bool {
	if listener.AllowedRoutes == nil { return namespace == gateway.Namespace }
	if len(listener.AllowedRoutes.Kinds) > 0 {
		found := false
		for _, kind := range listener.AllowedRoutes.Kinds { if kind.Kind == "HTTPRoute" && valueOr(kind.Group, "gateway.networking.k8s.io") == "gateway.networking.k8s.io" { found = true } }
		if !found { return false }
	}
	ns := listener.AllowedRoutes.Namespaces
	if ns == nil || ns.From == nil || *ns.From == gw.NamespacesFromSame { return namespace == gateway.Namespace }
	if *ns.From == gw.NamespacesFromAll { return true }
	if *ns.From != gw.NamespacesFromSelector || ns.Selector == nil { return false }
	resource := c.Inputs.Get(ctx, "Namespace", "", namespace); if resource == nil { return false }
	selector, err := metav1.LabelSelectorAsSelector(ns.Selector); if err != nil { return false }
	var meta struct { Metadata metav1.ObjectMeta `json:"metadata"` }
	if resource.Decode(&meta) != nil { return false }
	return selector.Matches(labels.Set(meta.Metadata.Labels))
}

func (c Compiler) referenceAllowed(ctx krt.HandlerContext, sourceKind, sourceNS, targetGroup, targetKind, targetNS, targetName string) bool {
	if sourceNS == targetNS { return true }
	sourceGroup := "gateway.networking.k8s.io"; if sourceKind == "TransitService" { sourceGroup = TransitGroup }
	for _, resource := range c.Inputs.List(ctx, "ReferenceGrant", targetNS) {
		var ref grant.ReferenceGrant; if resource.Decode(&ref) != nil { continue }
		from := false
		for _, item := range ref.Spec.From { if string(item.Group) == sourceGroup && string(item.Kind) == sourceKind && string(item.Namespace) == sourceNS { from = true } }
		if !from { continue }
		for _, item := range ref.Spec.To { if string(item.Group) == targetGroup && string(item.Kind) == targetKind && (item.Name == nil || string(*item.Name) == targetName) { return true } }
	}
	return false
}

func intersectHostnames(listener *gw.Hostname, routeHosts []gw.Hostname) []string {
	if len(routeHosts) == 0 { return []string{valueOr(listener, "*")} }
	result := []string{}
	for _, hostname := range routeHosts {
		host := strings.ToLower(string(hostname)); bound := strings.ToLower(valueOr(listener, "*"))
		if hostnameMatches(bound, host) { result = append(result, host) } else if hostnameMatches(host, bound) { result = append(result, bound) }
	}
	slices.Sort(result); return slices.Compact(result)
}
func hostnameMatches(pattern, host string) bool {
	return pattern == "*" || pattern == host || (strings.HasPrefix(pattern, "*.") && strings.HasSuffix(host, pattern[1:]) && len(host) > len(pattern)-1)
}
func valueOr[T ~string](p *T, fallback string) string { if p == nil { return fallback }; return string(*p) }

func deduplicateAgentConfig(config *route.AgentConfig) {
	providers := map[string]*route.AgentProvider{}; for _, x := range config.Providers { providers[x.Name] = x }; config.Providers = nil
	for _, key := range slices.Sorted(maps.Keys(providers)) { config.Providers = append(config.Providers, providers[key]) }
	backends := map[string]*route.AgentBackend{}; for _, x := range config.Backends { backends[x.Name] = x }; config.Backends = nil
	for _, key := range slices.Sorted(maps.Keys(backends)) { config.Backends = append(config.Backends, backends[key]) }
	policies := map[string]*route.AgentPolicy{}; for _, x := range config.Policies { policies[x.Name] = x }; config.Policies = nil
	for _, key := range slices.Sorted(maps.Keys(policies)) { config.Policies = append(config.Policies, policies[key]) }
}

func compareRoutes(a, b *route.AgentRoute) int {
	if len(a.Matches) == 0 || len(b.Matches) == 0 { return strings.Compare(a.Name, b.Name) }
	x, y := a.Matches[0], b.Matches[0]
	xExact, yExact := x.Path.GetExact() != "", y.Path.GetExact() != ""
	if xExact != yExact { if xExact { return -1 }; return 1 }
	xLength, yLength := len(x.Path.GetExact()+x.Path.GetPrefix()), len(y.Path.GetExact()+y.Path.GetPrefix())
	if xLength != yLength { return yLength-xLength }
	if (x.Method != "") != (y.Method != "") { if x.Method != "" { return -1 }; return 1 }
	if len(x.Headers) != len(y.Headers) { return len(y.Headers)-len(x.Headers) }
	return strings.Compare(a.Name,b.Name)
}

func unsupported(format string, args ...any) error { return fmt.Errorf(format, args...) }
