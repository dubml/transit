package controlplane

import (
	"encoding/base64"
	"fmt"
	"net"
	"net/url"
	"strconv"
	"strings"
	"time"

	route "github.com/kdubbo/xds-api/route/v1"
	"google.golang.org/protobuf/types/known/durationpb"
	"istio.io/istio/pkg/kube/krt"
	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	gw "sigs.k8s.io/gateway-api/apis/v1"
)

type SecretReference struct { Name string `json:"name"`; Namespace string `json:"namespace,omitempty"`; Key string `json:"key"` }
type ProviderSpec struct {
	Type string `json:"type,omitempty"`
	OpenAI *struct{} `json:"openai,omitempty"`
	Anthropic *struct{Model string `json:"model,omitempty"`} `json:"anthropic,omitempty"`
	Credential *SecretReference `json:"credential,omitempty"`
	RequestHeaders []Header `json:"requestHeaders,omitempty"`
}
type Header struct { Name string `json:"name"`; Value string `json:"value"` }
type LLMSpec struct {
	Endpoint string `json:"endpoint"`
	Provider ProviderSpec `json:"provider"`
	Models []string `json:"models,omitempty"`
	ModelRewrites map[string]string `json:"modelRewrites,omitempty"`
}
type TargetSpec struct {
	Name string `json:"name,omitempty"`
	Endpoint string `json:"endpoint,omitempty"`
	Host string `json:"host,omitempty"`
	BackendRef *struct { Name string `json:"name"`; Namespace string `json:"namespace,omitempty"` } `json:"backendRef,omitempty"`
	Port int32 `json:"port,omitempty"`
	Path string `json:"path,omitempty"`
	Tools []string `json:"tools,omitempty"`
	Agent string `json:"agent,omitempty"`
}
type PolicySpec struct {
	Auth *struct { Header string `json:"header"`; SecretRef SecretReference `json:"secretRef"` } `json:"auth,omitempty"`
	Timeout string `json:"timeout,omitempty"`
	Retry *struct { Attempts uint32 `json:"attempts"`; StatusCodes []uint32 `json:"statusCodes,omitempty"` } `json:"retry,omitempty"`
	MaxBodyBytes uint64 `json:"maxBodyBytes,omitempty"`
	RateLimit *struct { Requests uint32 `json:"requests"`; Window string `json:"window"`; Key string `json:"key,omitempty"`; Header string `json:"header,omitempty"` } `json:"rateLimit,omitempty"`
	TokenLimit *struct { Tokens uint64 `json:"tokens"`; Window string `json:"window"`; Key string `json:"key,omitempty"`; Header string `json:"header,omitempty"` } `json:"tokenLimit,omitempty"`
	RequestHeaders *struct { Add []Header `json:"add,omitempty"`; Remove []string `json:"remove,omitempty"` } `json:"requestHeaders,omitempty"`
	ResponseHeaders *struct { Add []Header `json:"add,omitempty"`; Remove []string `json:"remove,omitempty"` } `json:"responseHeaders,omitempty"`
}
type TransitService struct {
	metav1.TypeMeta `json:",inline"`
	metav1.ObjectMeta `json:"metadata,omitempty"`
	Spec struct {
		LLM *LLMSpec `json:"llm,omitempty"`
		AI *LLMSpec `json:"ai,omitempty"`
		MCP *struct { Targets []TargetSpec `json:"targets"` } `json:"mcp,omitempty"`
		A2A *TargetSpec `json:"a2a,omitempty"`
		Policies PolicySpec `json:"policies,omitempty"`
	} `json:"spec"`
}

func (c Compiler) compileHTTPRoute(ctx krt.HandlerContext, httpRoute gw.HTTPRoute, listener gw.Listener, hosts []string) (*route.AgentConfig, error) {
	result := &route.AgentConfig{}
	for ruleIndex, rule := range httpRoute.Spec.Rules {
		if len(rule.BackendRefs) == 0 { return nil, unsupported("rule %d has no backendRefs", ruleIndex) }
		name := fmt.Sprintf("%s/%s/%s/%d", httpRoute.Namespace, httpRoute.Name, listener.Name, ruleIndex)
		protocol := route.AgentProtocol_AGENT_PROTOCOL_UNSPECIFIED
		var weighted []*route.WeightedBackend
		for _, backendRef := range rule.BackendRefs {
			weight := int32(1); if backendRef.Weight != nil { weight = *backendRef.Weight }
			if weight < 0 { return nil, unsupported("negative backend weight") }; if weight == 0 { continue }
			if len(backendRef.Filters) > 0 { return nil, unsupported("backendRef filters are not supported") }
			ns := valueOr(backendRef.Namespace, httpRoute.Namespace)
			kind := valueOr(backendRef.Kind, "Service"); group := valueOr(backendRef.Group, "")
			if !c.referenceAllowed(ctx, "HTTPRoute", httpRoute.Namespace, group, kind, ns, string(backendRef.Name)) {
				return nil, unsupported("backend reference to %s/%s is not permitted by ReferenceGrant", ns, backendRef.Name)
			}
			var part *route.AgentConfig; var backendProtocol route.AgentProtocol; var err error
			switch {
			case group == "" && kind == "Service":
				if backendRef.Port == nil { return nil, unsupported("Service backend %s requires a port", backendRef.Name) }
				endpoint, err := c.serviceEndpoint(ctx, ns, string(backendRef.Name), int32(*backendRef.Port)); if err != nil { return nil, err }
				backendName := fmt.Sprintf("service/%s/%s/%d",ns,backendRef.Name,*backendRef.Port)
				part = &route.AgentConfig{Backends: []*route.AgentBackend{{Name:backendName, Backend:&route.AgentBackend_Http{Http:&route.HTTPBackend{Endpoint:endpoint}}}}}
				backendProtocol = route.AgentProtocol_HTTP
			case group == TransitGroup && kind == "TransitService":
				part, backendProtocol, err = c.compileTransitService(ctx, ns, string(backendRef.Name))
				if err != nil { return nil, err }
			default: return nil, unsupported("unsupported backend kind %s/%s", group, kind)
			}
			if protocol != route.AgentProtocol_AGENT_PROTOCOL_UNSPECIFIED && protocol != backendProtocol { return nil, unsupported("a rule cannot mix backend protocols") }
			protocol = backendProtocol
			result.Providers = append(result.Providers, part.Providers...); result.Backends = append(result.Backends, part.Backends...); result.Policies = append(result.Policies, part.Policies...)
			for _, backend := range part.Backends { weighted = append(weighted, &route.WeightedBackend{Name:backend.Name, Weight:uint32(weight)}) }
		}
		if len(weighted) == 0 { return nil, unsupported("rule %d has no positive-weight backends", ruleIndex) }
		policy, rewrite, err := compileFilters(name + "/filters", rule); if err != nil { return nil, err }
		var policies []string
		if policy != nil { result.Policies = append(result.Policies, policy); policies = append(policies, policy.Name) }
		matches := rule.Matches; if len(matches) == 0 { matches = []gw.HTTPRouteMatch{{}} }
		for matchIndex, match := range matches {
			if len(match.QueryParams) > 0 { return nil, unsupported("query parameter matching is not supported") }
			path := &route.AgentPathMatch{Match:&route.AgentPathMatch_Prefix{Prefix:"/"}}
			if match.Path != nil {
				value := valueOr(match.Path.Value, "/")
				switch valueOr(match.Path.Type, "PathPrefix") {
				case "PathPrefix": path.Match = &route.AgentPathMatch_Prefix{Prefix:value}
				case "Exact": path.Match = &route.AgentPathMatch_Exact{Exact:value}
				default: return nil, unsupported("unsupported path match type")
				}
			}
			if rewrite != nil && path.GetPrefix() == "" { return nil, unsupported("ReplacePrefixMatch requires a PathPrefix match") }
			var headers []*route.AgentHeaderMatch
			for _, header := range match.Headers {
				if valueOr(header.Type, "Exact") != "Exact" { return nil, unsupported("unsupported header match type") }
				headers = append(headers, &route.AgentHeaderMatch{Name:string(header.Name), Value:header.Value})
			}
			for hostIndex, host := range hosts {
				result.AgentRoutes = append(result.AgentRoutes, &route.AgentRoute{
					Name:fmt.Sprintf("%s/%d/%d",name,matchIndex,hostIndex), Protocol:protocol, ListenerPorts:[]uint32{uint32(listener.Port)},
					Matches:[]*route.AgentRouteMatch{{Path:path,Host:host,Method:valueOr(match.Method,""),Headers:headers}},
					WeightedBackends:weighted,Policies:policies,Rewrite:rewrite,
				})
			}
		}
	}
	return result, nil
}

func (c Compiler) serviceEndpoint(ctx krt.HandlerContext, namespace, name string, port int32) (string, error) {
	resource := c.Inputs.Get(ctx,"Service",namespace,name)
	if resource == nil { return "", unsupported("Service %s/%s does not exist",namespace,name) }
	var service corev1.Service; if err := resource.Decode(&service); err != nil { return "", err }
	found := false
	for _, servicePort := range service.Spec.Ports {
		if servicePort.Port == port && (servicePort.Protocol == "" || servicePort.Protocol == corev1.ProtocolTCP) { found = true }
	}
	if !found { return "", unsupported("Service %s/%s has no TCP port %d",namespace,name,port) }
	host := name + "." + namespace + ".svc." + strings.TrimPrefix(c.ClusterDomain,"svc.")
	if service.Spec.ClusterIP != "" && service.Spec.ClusterIP != corev1.ClusterIPNone { host = service.Spec.ClusterIP }
	if service.Spec.Type == corev1.ServiceTypeExternalName { host = service.Spec.ExternalName }
	return "http://" + net.JoinHostPort(host, strconv.Itoa(int(port))), nil
}

func (c Compiler) compileTransitService(ctx krt.HandlerContext, namespace, name string) (*route.AgentConfig, route.AgentProtocol, error) {
	resource := c.Inputs.Get(ctx,"TransitService",namespace,name)
	if resource == nil { return nil, 0, unsupported("TransitService %s/%s does not exist",namespace,name) }
	var service TransitService; if err := resource.Decode(&service); err != nil { return nil,0,err }
	key := "transit/" + namespace + "/" + name
	result := &route.AgentConfig{}
	policy, err := c.compilePolicy(ctx,namespace,key+"/policy",service.Spec.Policies)
	if err != nil { return nil,0,err }
	result.Policies = append(result.Policies,policy)
	llm := service.Spec.LLM; if llm == nil { llm = service.Spec.AI }
	count := 0; if llm != nil { count++ }; if service.Spec.MCP != nil { count++ }; if service.Spec.A2A != nil { count++ }
	if count != 1 || (service.Spec.LLM != nil && service.Spec.AI != nil) { return nil,0,unsupported("TransitService must select exactly one of llm, ai, mcp or a2a") }
	if llm != nil {
		if err := validateEndpoint(llm.Endpoint); err != nil { return nil,0,err }
		provider := &route.AgentProvider{Name:key+"/provider",BaseUrl:llm.Endpoint}
		switch {
		case llm.Provider.Type == "openai" || llm.Provider.OpenAI != nil: provider.Kind = route.AgentProviderKind_OPENAI
		case llm.Provider.Type == "anthropic" || llm.Provider.Anthropic != nil: provider.Kind = route.AgentProviderKind_ANTHROPIC
		default: return nil,0,unsupported("LLM provider must select openai or anthropic")
		}
		if llm.Provider.Credential != nil {
			provider.Credential, err = c.secretReference(ctx,namespace,*llm.Provider.Credential)
			if err != nil { return nil,0,err }
		}
		for _, h := range llm.Provider.RequestHeaders { provider.RequestHeaders = append(provider.RequestHeaders,&route.AgentHeaderValue{Name:h.Name,Value:h.Value}) }
		result.Providers = append(result.Providers,provider)
		result.Backends = append(result.Backends,&route.AgentBackend{Name:key,Policies:[]string{policy.Name},Backend:&route.AgentBackend_Llm{Llm:&route.LLMBackend{Provider:provider.Name,Models:llm.Models,Endpoint:llm.Endpoint,ModelRewrites:llm.ModelRewrites}}})
		return result,route.AgentProtocol_LLM,nil
	}
	if service.Spec.MCP != nil {
		if len(service.Spec.MCP.Targets) == 0 { return nil,0,unsupported("MCP requires at least one target") }
		for i,target := range service.Spec.MCP.Targets {
			endpoint,err := c.targetEndpoint(ctx,namespace,target); if err != nil { return nil,0,err }
			result.Backends = append(result.Backends,&route.AgentBackend{Name:fmt.Sprintf("%s/%d",key,i),Policies:[]string{policy.Name},Backend:&route.AgentBackend_Mcp{Mcp:&route.MCPBackend{Endpoint:endpoint,Tools:target.Tools}}})
		}
		return result,route.AgentProtocol_MCP,nil
	}
	endpoint,err := c.targetEndpoint(ctx,namespace,*service.Spec.A2A); if err != nil { return nil,0,err }
	result.Backends = append(result.Backends,&route.AgentBackend{Name:key,Policies:[]string{policy.Name},Backend:&route.AgentBackend_A2A{A2A:&route.A2ABackend{Endpoint:endpoint,Agent:service.Spec.A2A.Agent}}})
	return result,route.AgentProtocol_A2A,nil
}

func (c Compiler) targetEndpoint(ctx krt.HandlerContext, namespace string, target TargetSpec) (string,error) {
	if target.Endpoint != "" { return target.Endpoint,validateEndpoint(target.Endpoint) }
	if target.Port < 1 || target.Port > 65535 { return "",unsupported("target port must be between 1 and 65535") }
	var endpoint string
	if target.BackendRef != nil {
		ns := target.BackendRef.Namespace; if ns == "" { ns = namespace }
		if !c.referenceAllowed(ctx,"TransitService",namespace,"","Service",ns,target.BackendRef.Name) { return "",unsupported("TransitService backendRef is not permitted by ReferenceGrant") }
		var err error; endpoint,err = c.serviceEndpoint(ctx,ns,target.BackendRef.Name,target.Port); if err != nil { return "",err }
	} else {
		if target.Host == "" { return "",unsupported("target requires endpoint, host, or backendRef") }
		endpoint = "http://"+net.JoinHostPort(target.Host,strconv.Itoa(int(target.Port)))
	}
	if target.Path != "" { endpoint += "/"+strings.TrimPrefix(target.Path,"/") }
	return endpoint,validateEndpoint(endpoint)
}

func validateEndpoint(endpoint string) error {
	parsed,err := url.Parse(endpoint)
	if err != nil || parsed.Host == "" || (parsed.Scheme != "http" && parsed.Scheme != "https") || parsed.User != nil || parsed.Fragment != "" { return unsupported("endpoint must be an HTTP(S) URL without embedded credentials or fragment") }
	return nil
}

func (c Compiler) secretReference(ctx krt.HandlerContext, namespace string, ref SecretReference) (*route.SecretKeyReference,error) {
	if ref.Namespace != "" && ref.Namespace != namespace { return nil,unsupported("credential Secret must be in the TransitService namespace") }
	resource := c.Inputs.Get(ctx,"Secret",namespace,ref.Name); if resource == nil { return nil,unsupported("credential Secret %s/%s does not exist",namespace,ref.Name) }
	value,found,_ := unstructured.NestedString(resource.Object,"data",ref.Key)
	if !found { return nil,unsupported("credential Secret %s/%s has no requested key",namespace,ref.Name) }
	decoded,err := base64.StdEncoding.DecodeString(value); if err != nil || len(decoded)==0 { return nil,unsupported("credential Secret contains an invalid or empty value") }
	return &route.SecretKeyReference{Namespace:namespace,Name:ref.Name,Key:ref.Key},nil
}

func (c Compiler) compilePolicy(ctx krt.HandlerContext, namespace,name string,spec PolicySpec) (*route.AgentPolicy,error) {
	p := &route.AgentPolicy{Name:name,MaxBodyBytes:spec.MaxBodyBytes}
	var err error
	if spec.Timeout != "" { p.Timeout,err = positiveDuration(spec.Timeout); if err != nil { return nil,err } }
	if spec.Auth != nil {
		ref,err := c.secretReference(ctx,namespace,spec.Auth.SecretRef); if err != nil { return nil,err }
		if spec.Auth.Header == "" { return nil,unsupported("auth.header is required") }
		p.Auth = &route.ClientAuthPolicy{Header:spec.Auth.Header,SecretRef:ref}
	}
	if spec.Retry != nil { p.Retry = &route.AgentRetryPolicy{Attempts:spec.Retry.Attempts,StatusCodes:spec.Retry.StatusCodes} }
	if spec.RateLimit != nil {
		window,err := positiveDuration(spec.RateLimit.Window); if err != nil { return nil,err }
		key,err := rateLimitKey(spec.RateLimit.Key,spec.RateLimit.Header); if err != nil { return nil,err }
		if spec.RateLimit.Requests == 0 { return nil,unsupported("rateLimit.requests must be positive") }
		p.RateLimit = &route.AgentRateLimitPolicy{Requests:spec.RateLimit.Requests,Window:window,Key:key,Header:spec.RateLimit.Header}
	}
	if spec.TokenLimit != nil {
		window,err := positiveDuration(spec.TokenLimit.Window); if err != nil { return nil,err }
		key,err := rateLimitKey(spec.TokenLimit.Key,spec.TokenLimit.Header); if err != nil { return nil,err }
		if spec.TokenLimit.Tokens == 0 { return nil,unsupported("tokenLimit.tokens must be positive") }
		p.TokenLimit = &route.AgentTokenLimitPolicy{Tokens:spec.TokenLimit.Tokens,Window:window,Key:key,Header:spec.TokenLimit.Header}
	}
	if spec.RequestHeaders != nil { p.RequestHeaders = headerTransform(spec.RequestHeaders.Add,spec.RequestHeaders.Remove) }
	if spec.ResponseHeaders != nil { p.ResponseHeaders = headerTransform(spec.ResponseHeaders.Add,spec.ResponseHeaders.Remove) }
	return p,nil
}
func headerTransform(headers []Header, remove []string) *route.AgentHeaderTransform {
	x := &route.AgentHeaderTransform{Remove:remove}; for _, h := range headers { x.Add = append(x.Add,&route.AgentHeaderValue{Name:h.Name,Value:h.Value}) }; return x
}
func rateLimitKey(key,header string) (route.AgentRateLimitKey,error) {
	switch key { case "","route": return route.AgentRateLimitKey_ROUTE,nil; case "backend":return route.AgentRateLimitKey_BACKEND,nil; case "header": if header != "" { return route.AgentRateLimitKey_HEADER,nil } }
	return 0,unsupported("limit key must be route, backend, or header with a header name")
}
func positiveDuration(value string) (*durationpb.Duration,error) {
	d,err := time.ParseDuration(value); if err != nil || d <= 0 { return nil,unsupported("duration must be positive: %q",value) }; return durationpb.New(d),nil
}

func compileFilters(name string, rule gw.HTTPRouteRule) (*route.AgentPolicy,*route.PathRewrite,error) {
	p := &route.AgentPolicy{Name:name}; used := false
	var rewrite *route.PathRewrite
	for _, filter := range rule.Filters {
		switch filter.Type {
		case gw.HTTPRouteFilterRequestHeaderModifier,gw.HTTPRouteFilterResponseHeaderModifier:
			headers := filter.RequestHeaderModifier; if filter.Type == gw.HTTPRouteFilterResponseHeaderModifier { headers = filter.ResponseHeaderModifier }
			if headers == nil { return nil,nil,unsupported("header modifier has no configuration") }
			x := &route.AgentHeaderTransform{Remove:headers.Remove}
			for _, h := range headers.Set { x.Remove = append(x.Remove,string(h.Name)); x.Add = append(x.Add,&route.AgentHeaderValue{Name:string(h.Name),Value:h.Value}) }
			for _, h := range headers.Add { x.Add = append(x.Add,&route.AgentHeaderValue{Name:string(h.Name),Value:h.Value}) }
			if filter.Type == gw.HTTPRouteFilterResponseHeaderModifier { p.ResponseHeaders = x } else { p.RequestHeaders = x }; used = true
		case gw.HTTPRouteFilterURLRewrite:
			if rewrite != nil || filter.URLRewrite == nil || filter.URLRewrite.Hostname != nil || filter.URLRewrite.Path == nil || filter.URLRewrite.Path.Type != gw.PrefixMatchHTTPPathModifier || filter.URLRewrite.Path.ReplacePrefixMatch == nil { return nil,nil,unsupported("URLRewrite supports one ReplacePrefixMatch path modifier") }
			rewrite = &route.PathRewrite{ReplacePrefixMatch:*filter.URLRewrite.Path.ReplacePrefixMatch}
		default: return nil,nil,unsupported("unsupported HTTPRoute filter %s",filter.Type)
		}
	}
	if rule.Timeouts != nil && rule.Timeouts.Request != nil && string(*rule.Timeouts.Request) != "0s" {
		var err error; p.Timeout,err = positiveDuration(string(*rule.Timeouts.Request)); if err != nil { return nil,nil,err }; used = true
	}
	if !used { p = nil }
	return p,rewrite,nil
}
