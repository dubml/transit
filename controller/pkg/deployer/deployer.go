package controlplane

import (
	"context"
	"crypto/x509"
	"encoding/json"
	"fmt"
	"net/url"
	"reflect"
	"slices"
	"strings"

	listener "github.com/dubml/xds-api/listener/v1"
	route "github.com/dubml/xds-api/route/v1"
	apps "k8s.io/api/apps/v1"
	core "k8s.io/api/core/v1"
	rbac "k8s.io/api/rbac/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/schema"
	"k8s.io/apimachinery/pkg/types"
	"k8s.io/apimachinery/pkg/util/intstr"
	"k8s.io/client-go/dynamic"
	gw "sigs.k8s.io/gateway-api/apis/v1"
)

const managedByLabel = "app.kubernetes.io/managed-by"
const gatewayUIDLabel = "transit.dev/gateway-uid"

type DeploymentOptions struct {
	Image       string
	XDSAddress  string
	RootCA      string
	Replicas    int32
	ServiceType core.ServiceType
}

type DeploymentResource struct {
	GVR    schema.GroupVersionResource
	Object *unstructured.Unstructured
}

func deploymentResource(group, version, resource string, object any) (DeploymentResource, error) {
	data, err := runtime.DefaultUnstructuredConverter.ToUnstructured(object)
	if err != nil {
		return DeploymentResource{}, err
	}
	u := &unstructured.Unstructured{Object: data}
	return DeploymentResource{GVR: schema.GroupVersionResource{Group: group, Version: version, Resource: resource}, Object: u}, nil
}

func (options DeploymentOptions) Validate() error {
	if strings.TrimSpace(options.Image) == "" {
		return fmt.Errorf("data plane image is required")
	}
	endpoint, err := url.Parse(options.XDSAddress)
	if err != nil || endpoint.Scheme != "https" || endpoint.Hostname() == "" || endpoint.User != nil || endpoint.RawQuery != "" || endpoint.Fragment != "" || (endpoint.Path != "" && endpoint.Path != "/") {
		return fmt.Errorf("data plane requires an https xDS server address without credentials, path or query")
	}
	if !x509.NewCertPool().AppendCertsFromPEM([]byte(options.RootCA)) {
		return fmt.Errorf("data plane root CA must contain a PEM certificate")
	}
	if options.Replicas < 1 {
		return fmt.Errorf("data plane replicas must be positive")
	}
	if options.ServiceType != "" && options.ServiceType != core.ServiceTypeClusterIP && options.ServiceType != core.ServiceTypeLoadBalancer && options.ServiceType != core.ServiceTypeNodePort {
		return fmt.Errorf("unsupported data plane Service type %q", options.ServiceType)
	}
	return nil
}

func DesiredGatewayResources(gateway gw.Gateway, output GatewayOutput, options DeploymentOptions) ([]DeploymentResource, error) {
	if gateway.UID == "" {
		return nil, fmt.Errorf("Gateway UID is required for resource ownership")
	}
	if err := options.Validate(); err != nil {
		return nil, err
	}
	if options.ServiceType == "" {
		options.ServiceType = core.ServiceTypeLoadBalancer
	}
	name := DataPlaneName(gateway.Name)
	labels := map[string]string{managedByLabel: "transit-controller", gatewayUIDLabel: string(gateway.UID), "app.kubernetes.io/name": "transit"}
	owner := *metav1.NewControllerRef(&gateway, gw.SchemeGroupVersion.WithKind("Gateway"))
	metadata := metav1.ObjectMeta{Name: name, Namespace: gateway.Namespace, Labels: labels, OwnerReferences: []metav1.OwnerReference{owner}}
	portSet := map[int32]struct{}{}
	for _, encoded := range output.Resources[ListenerType] {
		value := &listener.Listener{}
		if err := encoded.Value.UnmarshalTo(value); err != nil {
			return nil, err
		}
		port := int32(value.GetAddress().GetSocketAddress().GetPortValue())
		if port < 1 || port > 65535 || port == 26021 {
			return nil, fmt.Errorf("invalid or reserved Gateway listener port %d", port)
		}
		portSet[port] = struct{}{}
	}
	// A temporary certificate failure must not release an allocated NodePort.
	// The xDS listener still controls whether the proxy can accept traffic.
	for _, declared := range gateway.Spec.Listeners {
		if declared.Protocol != gw.HTTPProtocolType && declared.Protocol != gw.HTTPSProtocolType {
			continue
		}
		port := int32(declared.Port)
		if port < 1 || port > 65535 || port == 26021 {
			return nil, fmt.Errorf("invalid or reserved Gateway listener port %d", port)
		}
		portSet[port] = struct{}{}
	}
	ports := make([]int32, 0, len(portSet))
	for port := range portSet {
		ports = append(ports, port)
	}
	slices.Sort(ports)
	if len(ports) == 0 {
		return nil, nil
	}
	var containerPorts []core.ContainerPort
	var servicePorts []core.ServicePort
	for _, port := range ports {
		portName := fmt.Sprintf("p-%d", port)
		containerPorts = append(containerPorts, core.ContainerPort{Name: portName, ContainerPort: port, Protocol: core.ProtocolTCP})
		servicePorts = append(servicePorts, core.ServicePort{Name: portName, Port: port, TargetPort: intstr.FromInt32(port), Protocol: core.ProtocolTCP})
	}
	containerPorts = append(containerPorts, core.ContainerPort{Name: "management", ContainerPort: 26021, Protocol: core.ProtocolTCP})
	secretNames := map[string]struct{}{}
	for _, encoded := range output.Resources[RouteType] {
		config := &route.RouteConfiguration{}
		if err := encoded.Value.UnmarshalTo(config); err != nil {
			return nil, err
		}
		refs := []*route.SecretKeyReference{}
		for _, provider := range config.GetAgentConfig().GetProviders() {
			if provider.Credential != nil {
				refs = append(refs, provider.Credential)
			}
		}
		for _, policy := range config.GetAgentConfig().GetPolicies() {
			if policy.GetAuth().GetSecretRef() != nil {
				refs = append(refs, policy.GetAuth().GetSecretRef())
			}
		}
		for _, ref := range refs {
			if ref.Namespace != "" && ref.Namespace != gateway.Namespace {
				return nil, fmt.Errorf("cross-namespace Secret deployment requires an explicit grant for %s/%s", ref.Namespace, ref.Name)
			}
			secretNames[ref.Name] = struct{}{}
		}
	}
	keys := make([]string, 0, len(secretNames))
	for key := range secretNames {
		keys = append(keys, key)
	}
	slices.Sort(keys)
	serviceAccount := &core.ServiceAccount{TypeMeta: metav1.TypeMeta{APIVersion: "v1", Kind: "ServiceAccount"}, ObjectMeta: metadata}
	caMap := &core.ConfigMap{TypeMeta: metav1.TypeMeta{APIVersion: "v1", Kind: "ConfigMap"}, ObjectMeta: metadata, Data: map[string]string{"ca.crt": options.RootCA}}
	role := &rbac.Role{TypeMeta: metav1.TypeMeta{APIVersion: "rbac.authorization.k8s.io/v1", Kind: "Role"}, ObjectMeta: metadata}
	if len(keys) > 0 {
		role.Rules = []rbac.PolicyRule{{APIGroups: []string{""}, Resources: []string{"secrets"}, ResourceNames: keys, Verbs: []string{"get"}}}
	}
	binding := &rbac.RoleBinding{TypeMeta: metav1.TypeMeta{APIVersion: "rbac.authorization.k8s.io/v1", Kind: "RoleBinding"}, ObjectMeta: metadata, RoleRef: rbac.RoleRef{APIGroup: "rbac.authorization.k8s.io", Kind: "Role", Name: name}, Subjects: []rbac.Subject{{Kind: "ServiceAccount", Namespace: gateway.Namespace, Name: name}}}
	var expiration int64 = 3600
	var rootMode int32 = 0444
	var uid int64 = 65532
	var yes = true
	var no = false
	probe := func(path string) *core.Probe {
		return &core.Probe{ProbeHandler: core.ProbeHandler{HTTPGet: &core.HTTPGetAction{Path: path, Port: intstr.FromString("management")}}, PeriodSeconds: 5, TimeoutSeconds: 2, FailureThreshold: 3}
	}
	deployment := &apps.Deployment{
		TypeMeta: metav1.TypeMeta{APIVersion: "apps/v1", Kind: "Deployment"}, ObjectMeta: metadata,
		Spec: apps.DeploymentSpec{Replicas: &options.Replicas, Selector: &metav1.LabelSelector{MatchLabels: labels},
			Template: core.PodTemplateSpec{ObjectMeta: metav1.ObjectMeta{Labels: labels}, Spec: core.PodSpec{
				ServiceAccountName: name, SecurityContext: &core.PodSecurityContext{RunAsNonRoot: &yes, RunAsUser: &uid, SeccompProfile: &core.SeccompProfile{Type: core.SeccompProfileTypeRuntimeDefault}},
				Containers: []core.Container{{Name: "transit", Image: options.Image, Ports: containerPorts,
					Args:           []string{"--mode=kubernetes", "--gateway=" + gateway.Namespace + "/" + gateway.Name, "--xds-address=" + options.XDSAddress, "--xds-root-ca=/etc/transit/xds/ca.crt", "--xds-token-file=/var/run/secrets/transit/xds/token", fmt.Sprintf("--http-addr=0.0.0.0:%d", ports[0]), "--ui-addr=0.0.0.0:26021"},
					Env:            []core.EnvVar{{Name: "POD_NAMESPACE", ValueFrom: &core.EnvVarSource{FieldRef: &core.ObjectFieldSelector{APIVersion: "v1", FieldPath: "metadata.namespace"}}}, {Name: "POD_NAME", ValueFrom: &core.EnvVarSource{FieldRef: &core.ObjectFieldSelector{APIVersion: "v1", FieldPath: "metadata.name"}}}, {Name: "TRANSIT_GATEWAY_NAME", Value: gateway.Name}},
					ReadinessProbe: probe("/readyz"), LivenessProbe: probe("/healthz"), StartupProbe: &core.Probe{ProbeHandler: core.ProbeHandler{HTTPGet: &core.HTTPGetAction{Path: "/healthz", Port: intstr.FromString("management")}}, PeriodSeconds: 2, FailureThreshold: 60},
					SecurityContext: &core.SecurityContext{AllowPrivilegeEscalation: &no, ReadOnlyRootFilesystem: &yes, Capabilities: &core.Capabilities{Drop: []core.Capability{"ALL"}, Add: []core.Capability{"NET_BIND_SERVICE"}}},
					VolumeMounts:    []core.VolumeMount{{Name: "xds-ca", MountPath: "/etc/transit/xds", ReadOnly: true}, {Name: "xds-token", MountPath: "/var/run/secrets/transit/xds", ReadOnly: true}},
				}},
				Volumes: []core.Volume{{Name: "xds-ca", VolumeSource: core.VolumeSource{ConfigMap: &core.ConfigMapVolumeSource{LocalObjectReference: core.LocalObjectReference{Name: name}}}}, {Name: "xds-token", VolumeSource: core.VolumeSource{Projected: &core.ProjectedVolumeSource{DefaultMode: &rootMode, Sources: []core.VolumeProjection{{ServiceAccountToken: &core.ServiceAccountTokenProjection{Audience: XDSAudience, ExpirationSeconds: &expiration, Path: "token"}}}}}}},
			}},
		},
	}
	service := &core.Service{TypeMeta: metav1.TypeMeta{APIVersion: "v1", Kind: "Service"}, ObjectMeta: metadata, Spec: core.ServiceSpec{Selector: labels, Ports: servicePorts, Type: options.ServiceType}}
	var result []DeploymentResource
	for _, entry := range []struct {
		group, version, resource string
		object                   any
	}{{"", "v1", "serviceaccounts", serviceAccount}, {"", "v1", "configmaps", caMap}, {"rbac.authorization.k8s.io", "v1", "roles", role}, {"rbac.authorization.k8s.io", "v1", "rolebindings", binding}, {"apps", "v1", "deployments", deployment}, {"", "v1", "services", service}} {
		value, err := deploymentResource(entry.group, entry.version, entry.resource, entry.object)
		if err != nil {
			return nil, err
		}
		result = append(result, value)
	}
	return result, nil
}

func ApplyGatewayResources(ctx context.Context, client dynamic.Interface, gateway gw.Gateway, resources []DeploymentResource) error {
	for _, desired := range resources {
		api := client.Resource(desired.GVR).Namespace(desired.Object.GetNamespace())
		current, err := api.Get(ctx, desired.Object.GetName(), metav1.GetOptions{})
		if apierrors.IsNotFound(err) {
			if _, err := api.Create(ctx, desired.Object, metav1.CreateOptions{FieldManager: "transit-controller"}); err != nil {
				return err
			}
			continue
		}
		if err != nil {
			return err
		}
		owned := false
		for _, owner := range current.GetOwnerReferences() {
			if owner.UID == gateway.UID && owner.Kind == "Gateway" && owner.APIVersion == gw.SchemeGroupVersion.String() && owner.Controller != nil && *owner.Controller {
				owned = true
			}
		}
		if !owned {
			return fmt.Errorf("refusing to adopt %s %s/%s owned outside this Gateway", desired.Object.GetKind(), desired.Object.GetNamespace(), desired.Object.GetName())
		}
		patch := desired.Object.DeepCopy()
		metadata := patch.Object["metadata"].(map[string]any)
		for _, field := range []string{"creationTimestamp", "generation", "uid", "managedFields", "resourceVersion"} {
			delete(metadata, field)
		}
		if patch.GetKind() == "Service" {
			preserveServiceAllocations(current, patch)
		}
		if containsDesiredFields(current.Object, patch.Object) {
			continue
		}
		// Preserve API defaults and reject concurrent writes; lists replace removed ports.
		patch.SetResourceVersion(current.GetResourceVersion())
		data, err := json.Marshal(patch)
		if err != nil {
			return err
		}
		if _, err := api.Patch(ctx, desired.Object.GetName(), types.MergePatchType, data, metav1.PatchOptions{FieldManager: "transit-controller"}); err != nil {
			return err
		}
	}
	return nil
}

func containsDesiredFields(current, desired any) bool {
	switch expected := desired.(type) {
	case map[string]any:
		actual, ok := current.(map[string]any)
		if !ok {
			return false
		}
		for key, value := range expected {
			if !containsDesiredFields(actual[key], value) {
				return false
			}
		}
		return true
	case []any:
		actual, ok := current.([]any)
		if !ok || len(actual) != len(expected) {
			return false
		}
		for i := range expected {
			if !containsDesiredFields(actual[i], expected[i]) {
				return false
			}
		}
		return true
	default:
		return reflect.DeepEqual(current, desired)
	}
}

func preserveServiceAllocations(current, desired *unstructured.Unstructured) {
	spec := desired.Object["spec"].(map[string]any)
	serviceType, _, _ := unstructured.NestedString(desired.Object, "spec", "type")
	if serviceType == string(core.ServiceTypeClusterIP) {
		for _, field := range []string{"healthCheckNodePort", "externalTrafficPolicy", "allocateLoadBalancerNodePorts", "loadBalancerClass"} {
			spec[field] = nil
		}
		return
	}
	if serviceType != string(core.ServiceTypeLoadBalancer) {
		for _, field := range []string{"healthCheckNodePort", "allocateLoadBalancerNodePorts", "loadBalancerClass"} {
			spec[field] = nil
		}
	}
	previous, _, _ := unstructured.NestedSlice(current.Object, "spec", "ports")
	ports, _, _ := unstructured.NestedSlice(desired.Object, "spec", "ports")
	for _, entry := range ports {
		port := entry.(map[string]any)
		for _, entry := range previous {
			old := entry.(map[string]any)
			if port["name"] == old["name"] && port["port"] == old["port"] && port["protocol"] == old["protocol"] && old["nodePort"] != nil {
				port["nodePort"] = old["nodePort"]
			}
		}
	}
	_ = unstructured.SetNestedSlice(desired.Object, ports, "spec", "ports")
}
