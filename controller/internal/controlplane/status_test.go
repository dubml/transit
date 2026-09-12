package controlplane

import (
	"context"
	"reflect"
	"testing"

	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/schema"
	"k8s.io/client-go/dynamic/fake"
)

func TestRouteStatusOwnershipAndWithdrawal(t *testing.T) {
	parent := func(controller string) map[string]any {
		return map[string]any{
			"parentRef":      map[string]any{"name": "edge"},
			"controllerName": controller,
			"conditions": []any{map[string]any{
				"type": "Accepted", "status": "True", "reason": "Accepted",
				"message": "Accepted", "observedGeneration": int64(1),
				"lastTransitionTime": "2026-09-12T00:00:00Z",
			}},
		}
	}
	foreign := parent("example.com/other-controller")
	ours := parent(ControllerName)
	for _, test := range []struct {
		name       string
		status     map[string]any
		wantWrites int
		want       []any
	}{
		{name: "unmanaged without status"},
		{name: "unmanaged empty status", status: map[string]any{}},
		{name: "foreign controller untouched", status: map[string]any{"parents": []any{foreign}}},
		{name: "withdraw last managed parent", status: map[string]any{"parents": []any{ours}}, wantWrites: 1, want: []any{}},
		{name: "withdraw ours and preserve foreign parent", status: map[string]any{"parents": []any{foreign, ours}}, wantWrites: 1, want: []any{foreign}},
	} {
		t.Run(test.name, func(t *testing.T) {
			object := &unstructured.Unstructured{Object: map[string]any{
				"apiVersion": "gateway.networking.k8s.io/v1", "kind": "HTTPRoute",
				"metadata": map[string]any{"name": "route", "namespace": "team", "generation": int64(1)},
			}}
			if test.status != nil {
				object.Object["status"] = test.status
			}
			client := fake.NewSimpleDynamicClient(runtime.NewScheme(), object.DeepCopy())
			err := WriteResourceStatuses(context.Background(), client, []StatusUpdate{{Kind: "HTTPRoute", Namespace: "team", Name: "route", Generation: 1}}, false, "")
			if err != nil {
				t.Fatal(err)
			}
			writes := 0
			for _, action := range client.Actions() {
				if action.GetVerb() == "update" && action.GetSubresource() == "status" {
					writes++
				}
			}
			if writes != test.wantWrites {
				t.Fatalf("status writes = %d, want %d", writes, test.wantWrites)
			}
			resource := schema.GroupVersionResource{Group: "gateway.networking.k8s.io", Version: "v1", Resource: "httproutes"}
			actual, err := client.Resource(resource).Namespace("team").Get(context.Background(), "route", metav1.GetOptions{})
			if err != nil {
				t.Fatal(err)
			}
			if test.wantWrites == 0 {
				if !reflect.DeepEqual(actual.Object, object.Object) {
					t.Fatal("unmanaged route changed")
				}
				return
			}
			parents, found, err := unstructured.NestedSlice(actual.Object, "status", "parents")
			if err != nil || !found || parents == nil || !reflect.DeepEqual(parents, test.want) {
				t.Fatalf("parent status = %#v, found=%v, err=%v, want=%#v", parents, found, err, test.want)
			}
		})
	}
}
