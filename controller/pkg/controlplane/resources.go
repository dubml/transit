package controlplane

import (
	"encoding/json"
	"reflect"
	"strings"

	"istio.io/istio/pkg/kube/krt"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime/schema"
)

const (
	ControllerName = "transit.dev/gateway-controller"
	TransitGroup = "networking.dubbo.apache.org"
	TransitVersion = "v1alpha3"
)

type ResourceType struct {
	Kind string
	GVR schema.GroupVersionResource
	Namespaced bool
}

var ResourceTypes = []ResourceType{
	{"GatewayClass", schema.GroupVersionResource{Group: "gateway.networking.k8s.io", Version: "v1", Resource: "gatewayclasses"}, false},
	{"Gateway", schema.GroupVersionResource{Group: "gateway.networking.k8s.io", Version: "v1", Resource: "gateways"}, true},
	{"HTTPRoute", schema.GroupVersionResource{Group: "gateway.networking.k8s.io", Version: "v1", Resource: "httproutes"}, true},
	{"ReferenceGrant", schema.GroupVersionResource{Group: "gateway.networking.k8s.io", Version: "v1beta1", Resource: "referencegrants"}, true},
	{"TransitService", schema.GroupVersionResource{Group: TransitGroup, Version: TransitVersion, Resource: "transitservices"}, true},
	{"Service", schema.GroupVersionResource{Version: "v1", Resource: "services"}, true},
	{"Secret", schema.GroupVersionResource{Version: "v1", Resource: "secrets"}, true},
	{"Namespace", schema.GroupVersionResource{Version: "v1", Resource: "namespaces"}, false},
	{"EndpointSlice", schema.GroupVersionResource{Group: "discovery.k8s.io", Version: "v1", Resource: "endpointslices"}, true},
}

// Resource contains only fields relevant to compilation. Status writes must not
// trigger another compilation solely because resourceVersion changed.
type Resource struct {
	Kind string
	Namespace string
	Name string
	Object map[string]any
}

func NewResource(kind string, obj *unstructured.Unstructured) Resource {
	copy := obj.DeepCopy()
	delete(copy.Object, "status")
	metadata, _, _ := unstructured.NestedMap(copy.Object, "metadata")
	for _, key := range []string{"resourceVersion", "managedFields", "selfLink"} {
		delete(metadata, key)
	}
	copy.Object["metadata"] = metadata
	return Resource{Kind: kind, Namespace: obj.GetNamespace(), Name: obj.GetName(), Object: copy.Object}
}

func (r Resource) ResourceName() string { return resourceKey(r.Kind, r.Namespace, r.Name) }
func (r Resource) Equals(other Resource) bool { return reflect.DeepEqual(r, other) }
func (r Resource) Decode(target any) error {
	data, err := json.Marshal(r.Object)
	if err != nil { return err }
	return json.Unmarshal(data, target)
}
func resourceKey(kind, namespace, name string) string { return kind + "/" + namespace + "/" + name }
func gatewayKey(namespace, name string) string { return namespace + "/" + name }

type Inputs struct {
	Objects krt.StaticCollection[Resource]
	ByKindNamespace krt.Index[string, Resource]
	RoutesByGateway krt.Index[string, Resource]
}

func NewInputs(syncer krt.Syncer, stop <-chan struct{}) Inputs {
	objects := krt.NewStaticCollection[Resource](syncer, nil, krt.WithName("transit/inputs"), krt.WithStop(stop))
	return Inputs{
		Objects: objects,
		ByKindNamespace: krt.NewIndex(objects, "kind-namespace", func(r Resource) []string {
			return []string{r.Kind + "/" + r.Namespace}
		}),
		RoutesByGateway: krt.NewIndex(objects, "route-parent-gateway", func(r Resource) []string {
			if r.Kind != "HTTPRoute" { return nil }
			refs, _, _ := unstructured.NestedSlice(r.Object, "spec", "parentRefs")
			var keys []string
			for _, item := range refs {
				ref, ok := item.(map[string]any); if !ok { continue }
				group, _ := ref["group"].(string)
				kind, _ := ref["kind"].(string)
				if (group != "" && group != "gateway.networking.k8s.io") || (kind != "" && kind != "Gateway") { continue }
				ns, _ := ref["namespace"].(string); if ns == "" { ns = r.Namespace }
				name, _ := ref["name"].(string)
				if name != "" { keys = append(keys, gatewayKey(ns, name)) }
			}
			return keys
		}),
	}
}

func (in Inputs) Get(ctx krt.HandlerContext, kind, namespace, name string) *Resource {
	return krt.FetchOne(ctx, in.Objects, krt.FilterKey(resourceKey(kind, namespace, name)))
}
func (in Inputs) List(ctx krt.HandlerContext, kind, namespace string) []Resource {
	return krt.Fetch(ctx, in.Objects, krt.FilterIndex(in.ByKindNamespace, kind + "/" + namespace))
}

func resourceType(kind string) (ResourceType, bool) {
	for _, rt := range ResourceTypes { if strings.EqualFold(rt.Kind, kind) { return rt, true } }
	return ResourceType{}, false
}
