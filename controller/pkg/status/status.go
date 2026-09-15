package controlplane

import (
	"context"
	"encoding/json"
	"reflect"
	"slices"
	"strings"

	apierrors "k8s.io/apimachinery/pkg/api/errors"
	apiMeta "k8s.io/apimachinery/pkg/api/meta"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/client-go/dynamic"
	"k8s.io/client-go/util/retry"
	gw "sigs.k8s.io/gateway-api/apis/v1"
)

func statusConditions(desired, previous []metav1.Condition, programmed bool, message string) []metav1.Condition {
	conditions := append([]metav1.Condition(nil), previous...)
	for _, condition := range desired {
		if condition.Type == "Programmed" && condition.Status == metav1.ConditionTrue && !programmed {
			condition.Status = metav1.ConditionFalse
			condition.Reason = "Pending"
			condition.Message = message
		}
		apiMeta.SetStatusCondition(&conditions, condition)
	}
	return conditions
}

func parentKey(parent gw.ParentReference, namespace string) string {
	copy := parent
	group := gw.Group("gateway.networking.k8s.io")
	kind := gw.Kind("Gateway")
	ns := gw.Namespace(namespace)
	if copy.Group == nil {
		copy.Group = &group
	}
	if copy.Kind == nil {
		copy.Kind = &kind
	}
	if copy.Namespace == nil {
		copy.Namespace = &ns
	}
	data, _ := json.Marshal(copy)
	return string(data)
}

func WriteResourceStatuses(ctx context.Context, client dynamic.Interface, updates []StatusUpdate, programmed bool, message string) error {
	groups := map[string][]StatusUpdate{}
	for _, update := range updates {
		key := resourceKey(update.Kind, update.Namespace, update.Name)
		groups[key] = append(groups[key], update)
	}
	for _, records := range groups {
		first := records[0]
		slices.SortStableFunc(records, func(a, b StatusUpdate) int {
			if a.Parent == nil || b.Parent == nil {
				return 0
			}
			return strings.Compare(parentKey(*a.Parent, first.Namespace), parentKey(*b.Parent, first.Namespace))
		})
		var resourceType *ResourceType
		for i := range ResourceTypes {
			if ResourceTypes[i].Kind == first.Kind {
				resourceType = &ResourceTypes[i]
				break
			}
		}
		if resourceType == nil {
			continue
		}
		var api dynamic.ResourceInterface = client.Resource(resourceType.GVR)
		if resourceType.Namespaced {
			api = client.Resource(resourceType.GVR).Namespace(first.Namespace)
		}
		err := retry.RetryOnConflict(retry.DefaultRetry, func() error {
			object, err := api.Get(ctx, first.Name, metav1.GetOptions{})
			if apierrors.IsNotFound(err) {
				return nil
			}
			if err != nil {
				return err
			}
			if object.GetGeneration() != first.Generation {
				return nil
			}
			before, _, _ := unstructured.NestedMap(object.Object, "status")
			if before == nil {
				before = map[string]any{}
			}
			encoded, _ := json.Marshal(before)
			var next any
			switch first.Kind {
			case "GatewayClass":
				var current gw.GatewayClassStatus
				if err := json.Unmarshal(encoded, &current); err != nil {
					return err
				}
				current.Conditions = statusConditions(first.Conditions, current.Conditions, programmed, message)
				next = &current
			case "Gateway":
				var current gw.GatewayStatus
				if err := json.Unmarshal(encoded, &current); err != nil {
					return err
				}
				current.Conditions = statusConditions(first.Conditions, current.Conditions, programmed, message)
				listeners := append([]gw.ListenerStatus(nil), first.Listeners...)
				for i := range listeners {
					var old []metav1.Condition
					for _, previous := range current.Listeners {
						if previous.Name == listeners[i].Name {
							old = previous.Conditions
							break
						}
					}
					listeners[i].Conditions = statusConditions(listeners[i].Conditions, old, programmed, message)
				}
				current.Listeners = listeners
				if first.Addresses != nil {
					current.Addresses = *first.Addresses
				}
				next = &current
			case "HTTPRoute":
				var current gw.HTTPRouteStatus
				if err := json.Unmarshal(encoded, &current); err != nil {
					return err
				}
				ours := map[string]gw.RouteParentStatus{}
				for _, previous := range current.Parents {
					if string(previous.ControllerName) == ControllerName {
						ours[parentKey(previous.ParentRef, first.Namespace)] = previous
					}
				}
				if len(ours) == 0 && !slices.ContainsFunc(records, func(record StatusUpdate) bool { return record.Parent != nil }) {
					return nil
				}
				// Gateway API requires an array when withdrawing our last parent status.
				parents := make([]gw.RouteParentStatus, 0, len(current.Parents))
				for _, previous := range current.Parents {
					if string(previous.ControllerName) != ControllerName {
						parents = append(parents, previous)
					}
				}
				seen := map[string]bool{}
				for _, record := range records {
					if record.Parent == nil {
						continue
					}
					key := parentKey(*record.Parent, first.Namespace)
					previous := ours[key]
					conditions := record.Conditions
					if seen[key] {
						conditions = append([]metav1.Condition(nil), conditions...)
						for i := range conditions {
							if old := apiMeta.FindStatusCondition(previous.Conditions, conditions[i].Type); old != nil && old.Status == metav1.ConditionTrue {
								conditions[i] = *old
							}
						}
					}
					ours[key] = gw.RouteParentStatus{ParentRef: *record.Parent, ControllerName: gw.GatewayController(ControllerName), Conditions: statusConditions(conditions, previous.Conditions, programmed, message)}
					seen[key] = true
				}
				for _, record := range records {
					if record.Parent == nil {
						continue
					}
					key := parentKey(*record.Parent, first.Namespace)
					if seen[key] {
						parents = append(parents, ours[key])
						delete(seen, key)
					}
				}
				current.Parents = parents
				next = &current
			default:
				return nil
			}
			after, err := runtime.DefaultUnstructuredConverter.ToUnstructured(next)
			if err != nil {
				return err
			}
			if reflect.DeepEqual(before, after) {
				return nil
			}
			object.Object["status"] = after
			_, err = api.UpdateStatus(ctx, object, metav1.UpdateOptions{FieldManager: "transit-controller"})
			return err
		})
		if err != nil {
			return err
		}
	}
	return nil
}
