package controlplane

import (
	"context"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"istio.io/istio/pkg/kube/krt"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/schema"
	"k8s.io/client-go/dynamic"
	"k8s.io/client-go/dynamic/dynamicinformer"
	"k8s.io/client-go/tools/cache"
	"k8s.io/client-go/util/workqueue"
	gw "sigs.k8s.io/gateway-api/apis/v1"
)

var managedResourceTypes = []schema.GroupVersionResource{
	{Group: "apps", Version: "v1", Resource: "deployments"},
	{Version: "v1", Resource: "services"},
	{Version: "v1", Resource: "serviceaccounts"},
	{Version: "v1", Resource: "configmaps"},
	{Group: "rbac.authorization.k8s.io", Version: "v1", Resource: "roles"},
	{Group: "rbac.authorization.k8s.io", Version: "v1", Resource: "rolebindings"},
}

type Reconciler struct {
	client  dynamic.Interface
	inputs  Inputs
	outputs krt.Collection[GatewayOutput]
	options DeploymentOptions
	xds     *XDSServer
	queue   workqueue.TypedRateLimitingInterface[string]
	factory dynamicinformer.DynamicSharedInformerFactory
	ready   chan struct{}
}

func NewReconciler(client dynamic.Interface, inputs Inputs, outputs krt.Collection[GatewayOutput], options DeploymentOptions, xds *XDSServer) (*Reconciler, error) {
	r := &Reconciler{client: client, inputs: inputs, outputs: outputs, options: options, xds: xds, ready: make(chan struct{}), queue: workqueue.NewTypedRateLimitingQueue(workqueue.DefaultTypedControllerRateLimiter[string]())}
	r.factory = dynamicinformer.NewFilteredDynamicSharedInformerFactory(client, 0, metav1.NamespaceAll, func(options *metav1.ListOptions) { options.LabelSelector = managedByLabel + "=transit-controller" })
	notify := func(object any) {
		if deleted, ok := object.(cache.DeletedFinalStateUnknown); ok {
			object = deleted.Obj
		}
		if value, ok := object.(*unstructured.Unstructured); ok {
			for _, owner := range value.GetOwnerReferences() {
				if owner.Kind == "Gateway" && owner.APIVersion == gw.SchemeGroupVersion.String() {
					r.Enqueue(gatewayKey(value.GetNamespace(), owner.Name))
				}
			}
		}
	}
	for _, gvr := range managedResourceTypes {
		_, err := r.factory.ForResource(gvr).Informer().AddEventHandler(cache.ResourceEventHandlerFuncs{AddFunc: notify, UpdateFunc: func(_, next any) { notify(next) }, DeleteFunc: notify})
		if err != nil {
			r.queue.ShutDown()
			return nil, err
		}
	}
	outputs.Register(func(event krt.Event[GatewayOutput]) {
		r.Enqueue(event.Latest().Key)
		for _, value := range []*GatewayOutput{event.Old, event.New} {
			if value != nil {
				for _, update := range value.Statuses {
					if update.Kind == "HTTPRoute" {
						r.queue.Add("route/" + gatewayKey(update.Namespace, update.Name))
					}
				}
			}
		}
	})
	inputs.Objects.Register(func(event krt.Event[Resource]) {
		value := event.Latest()
		switch value.Kind {
		case "Gateway":
			r.Enqueue(gatewayKey(value.Namespace, value.Name))
		case "HTTPRoute":
			r.queue.Add("route/" + gatewayKey(value.Namespace, value.Name))
		case "GatewayClass":
			r.queue.Add("class/" + value.Name)
		}
	})
	xds.SetObserver(r.Enqueue)
	return r, nil
}

func (r *Reconciler) Enqueue(key string) { r.queue.Add("gateway/" + key) }

func (r *Reconciler) CheckAccess(ctx context.Context) error {
	for _, resource := range managedResourceTypes {
		if _, err := r.client.Resource(resource).List(ctx, metav1.ListOptions{Limit: 1, LabelSelector: managedByLabel + "=transit-controller"}); err != nil {
			return fmt.Errorf("access managed resource %s: %w", resource, err)
		}
	}
	return nil
}

func (r *Reconciler) Run(ctx context.Context) error {
	defer r.queue.ShutDown()
	go func() { <-ctx.Done(); r.queue.ShutDown() }()
	r.factory.Start(ctx.Done())
	for resource, synced := range r.factory.WaitForCacheSync(ctx.Done()) {
		if !synced {
			return fmt.Errorf("managed resource synchronization failed for %s", resource)
		}
	}
	if !r.outputs.WaitUntilSynced(ctx.Done()) {
		return fmt.Errorf("gateway derivation did not synchronize")
	}
	close(r.ready)
	for {
		key, shutdown := r.queue.Get()
		if shutdown {
			return nil
		}
		request, cancel := context.WithTimeout(ctx, 30*time.Second)
		err := r.reconcile(request, key)
		cancel()
		if err != nil {
			slog.Warn("Gateway reconciliation failed", "resource", key, "error", err)
			r.queue.AddRateLimited(key)
		} else {
			r.queue.Forget(key)
		}
		r.queue.Done(key)
	}
}

func (r *Reconciler) reconcile(ctx context.Context, key string) error {
	if strings.HasPrefix(key, "route/") {
		return r.reconcileRouteStatus(ctx, strings.TrimPrefix(key, "route/"))
	}
	if strings.HasPrefix(key, "class/") {
		value := r.inputs.Objects.GetKey(resourceKey("GatewayClass", "", strings.TrimPrefix(key, "class/")))
		if value == nil {
			return nil
		}
		var class gw.GatewayClass
		if err := value.Decode(&class); err != nil {
			return err
		}
		if string(class.Spec.ControllerName) != ControllerName {
			return nil
		}
		accepted := class.Spec.ParametersRef == nil
		reason, message := "Accepted", "GatewayClass is handled by the Transit controller"
		if !accepted {
			reason = "InvalidParameters"
			message = "GatewayClass parametersRef is not supported"
		}
		return WriteResourceStatuses(ctx, r.client, []StatusUpdate{{Kind: "GatewayClass", Name: class.Name, Generation: class.Generation, Conditions: []metav1.Condition{condition("Accepted", accepted, reason, message, class.Generation)}}}, false, "")
	}
	parts := strings.Split(strings.TrimPrefix(key, "gateway/"), "/")
	if len(parts) != 2 {
		return fmt.Errorf("invalid Gateway queue key")
	}
	value := r.inputs.Objects.GetKey(resourceKey("Gateway", parts[0], parts[1]))
	if value == nil {
		return nil
	}
	var gateway gw.Gateway
	if err := value.Decode(&gateway); err != nil {
		return err
	}
	output := r.outputs.GetKey(gatewayKey(gateway.Namespace, gateway.Name))
	if output == nil {
		return r.removeOwnedResources(ctx, gateway)
	}
	var err error
	if len(output.Resources[ListenerType]) == 0 {
		err = r.removeOwnedResources(ctx, gateway)
	} else {
		var desired []DeploymentResource
		desired, err = DesiredGatewayResources(gateway, *output, r.options)
		if err == nil {
			err = ApplyGatewayResources(ctx, r.client, gateway, desired)
		}
	}
	programmed := false
	var addresses []gw.GatewayStatusAddress
	message := "Waiting for the data plane deployment and xDS acknowledgements"
	if err != nil {
		message = err.Error()
	} else {
		var ready bool
		ready, err = r.deploymentReady(ctx, gateway)
		if err != nil {
			message = err.Error()
		}
		programmed = ready && r.xds.AcknowledgedReplicas(output.Key) >= int(r.options.Replicas)
		if err == nil {
			addresses, err = r.gatewayAddresses(ctx, gateway)
			if err != nil {
				message = err.Error()
			}
			if len(addresses) == 0 || err != nil {
				programmed = false
				if err == nil {
					message = "Waiting for the data plane Service address"
				}
			}
		}
	}
	var statuses []StatusUpdate
	for _, update := range output.Statuses {
		if update.Kind == "Gateway" {
			update.Addresses = &addresses
			statuses = append(statuses, update)
		}
	}
	if statusErr := WriteResourceStatuses(ctx, r.client, statuses, programmed, message); statusErr != nil {
		return statusErr
	}
	return err
}

func (r *Reconciler) reconcileRouteStatus(ctx context.Context, key string) error {
	parts := strings.Split(key, "/")
	if len(parts) != 2 {
		return fmt.Errorf("invalid Route queue key")
	}
	value := r.inputs.Objects.GetKey(resourceKey("HTTPRoute", parts[0], parts[1]))
	if value == nil {
		return nil
	}
	var route gw.HTTPRoute
	if err := value.Decode(&route); err != nil {
		return err
	}
	var statuses []StatusUpdate
	for _, output := range r.outputs.List() {
		for _, status := range output.Statuses {
			if status.Kind == "HTTPRoute" && status.Namespace == route.Namespace && status.Name == route.Name {
				statuses = append(statuses, status)
			}
		}
	}
	if len(statuses) == 0 {
		statuses = []StatusUpdate{{Kind: "HTTPRoute", Namespace: route.Namespace, Name: route.Name, Generation: route.Generation}}
	}
	return WriteResourceStatuses(ctx, r.client, statuses, false, "")
}

func (r *Reconciler) gatewayAddresses(ctx context.Context, gateway gw.Gateway) ([]gw.GatewayStatusAddress, error) {
	object, err := r.client.Resource(managedResourceTypes[1]).Namespace(gateway.Namespace).Get(ctx, DataPlaneName(gateway.Name), metav1.GetOptions{})
	if apierrors.IsNotFound(err) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	var service core.Service
	if err := runtime.DefaultUnstructuredConverter.FromUnstructured(object.Object, &service); err != nil {
		return nil, err
	}
	var addresses []gw.GatewayStatusAddress
	ip, hostname := gw.IPAddressType, gw.HostnameAddressType
	if service.Spec.Type == core.ServiceTypeLoadBalancer {
		for _, ingress := range service.Status.LoadBalancer.Ingress {
			if ingress.IP != "" {
				addresses = append(addresses, gw.GatewayStatusAddress{Type: &ip, Value: ingress.IP})
			}
			if ingress.Hostname != "" {
				addresses = append(addresses, gw.GatewayStatusAddress{Type: &hostname, Value: ingress.Hostname})
			}
		}
	} else {
		ips := service.Spec.ClusterIPs
		if len(ips) == 0 {
			ips = []string{service.Spec.ClusterIP}
		}
		for _, address := range ips {
			if address != "" && address != core.ClusterIPNone {
				addresses = append(addresses, gw.GatewayStatusAddress{Type: &ip, Value: address})
			}
		}
	}
	return addresses, nil
}

func (r *Reconciler) deploymentReady(ctx context.Context, gateway gw.Gateway) (bool, error) {
	deployment, err := r.client.Resource(managedResourceTypes[0]).Namespace(gateway.Namespace).Get(ctx, DataPlaneName(gateway.Name), metav1.GetOptions{})
	if apierrors.IsNotFound(err) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	observed, _, _ := unstructured.NestedInt64(deployment.Object, "status", "observedGeneration")
	available, _, _ := unstructured.NestedInt64(deployment.Object, "status", "availableReplicas")
	updated, _, _ := unstructured.NestedInt64(deployment.Object, "status", "updatedReplicas")
	return observed >= deployment.GetGeneration() && available >= int64(r.options.Replicas) && updated >= int64(r.options.Replicas), nil
}

func (r *Reconciler) removeOwnedResources(ctx context.Context, gateway gw.Gateway) error {
	for _, gvr := range managedResourceTypes {
		api := r.client.Resource(gvr).Namespace(gateway.Namespace)
		objects, err := api.List(ctx, metav1.ListOptions{LabelSelector: managedByLabel + "=transit-controller," + gatewayUIDLabel + "=" + string(gateway.UID)})
		if err != nil {
			return err
		}
		for _, object := range objects.Items {
			for _, owner := range object.GetOwnerReferences() {
				if owner.UID == gateway.UID && owner.Controller != nil && *owner.Controller {
					uid := object.GetUID()
					if err := api.Delete(ctx, object.GetName(), metav1.DeleteOptions{Preconditions: &metav1.Preconditions{UID: &uid}}); err != nil && !apierrors.IsNotFound(err) {
						return err
					}
					break
				}
			}
		}
	}
	return nil
}
