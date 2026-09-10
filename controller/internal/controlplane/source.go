package controlplane

import (
	"context"
	"fmt"
	"sync"

	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/client-go/dynamic"
	"k8s.io/client-go/dynamic/dynamicinformer"
	"k8s.io/client-go/tools/cache"
)

type initialSync struct { ready chan struct{} }
func (s *initialSync) HasSynced() bool { select { case <-s.ready: return true; default: return false } }
func (s *initialSync) WaitUntilSynced(stop <-chan struct{}) bool {
	select { case <-s.ready: return true; case <-stop: return false }
}

type KubernetesSource struct {
	Inputs Inputs
	client dynamic.Interface
	factory dynamicinformer.DynamicSharedInformerFactory
	sync *initialSync
	once sync.Once
}

func NewKubernetesSource(client dynamic.Interface, stop <-chan struct{}) (*KubernetesSource, error) {
	synced := &initialSync{ready: make(chan struct{})}
	s := &KubernetesSource{client: client, factory: dynamicinformer.NewDynamicSharedInformerFactory(client, 0), sync: synced}
	s.Inputs = NewInputs(synced, stop)
	for _, rt := range ResourceTypes {
		informer := s.factory.ForResource(rt.GVR).Informer()
		_, err := informer.AddEventHandler(cache.ResourceEventHandlerFuncs{
			AddFunc: func(obj any) { s.upsert(rt.Kind, obj) },
			UpdateFunc: func(_, obj any) { s.upsert(rt.Kind, obj) },
			DeleteFunc: func(obj any) {
				if tombstone, ok := obj.(cache.DeletedFinalStateUnknown); ok { obj = tombstone.Obj }
				if u, ok := obj.(*unstructured.Unstructured); ok {
					s.Inputs.Objects.DeleteObject(resourceKey(rt.Kind, u.GetNamespace(), u.GetName()))
				}
			},
		})
		if err != nil { return nil, fmt.Errorf("register %s informer: %w", rt.Kind, err) }
	}
	return s, nil
}

func (s *KubernetesSource) upsert(kind string, obj any) {
	if u, ok := obj.(*unstructured.Unstructured); ok {
		s.Inputs.Objects.ConditionalUpdateObject(NewResource(kind, u))
	}
}

func (s *KubernetesSource) Start(ctx context.Context) error {
	s.factory.Start(ctx.Done())
	for gvr, ok := range s.factory.WaitForCacheSync(ctx.Done()) {
		if !ok { return fmt.Errorf("initial Kubernetes resource synchronization failed for %s", gvr) }
	}
	s.once.Do(func() { close(s.sync.ready) })
	return nil
}

func (s *KubernetesSource) CheckAccess(ctx context.Context) error {
	for _, rt := range ResourceTypes {
		var api dynamic.ResourceInterface = s.client.Resource(rt.GVR)
		if rt.Namespaced { api = s.client.Resource(rt.GVR).Namespace(metav1.NamespaceAll) }
		if _, err := api.List(ctx, metav1.ListOptions{Limit: 1}); err != nil {
			return fmt.Errorf("cannot read required resource %s: %w", rt.Kind, err)
		}
	}
	return nil
}
