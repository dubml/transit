package controlplane

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"io"
	"maps"
	"slices"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"

	core "github.com/dubml/xds-api/core/v1"
	discovery "github.com/dubml/xds-api/service/discovery/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"istio.io/istio/pkg/kube/krt"
)

const (
	ListenerType = "type.googleapis.com/listener.v1.Listener"
	RouteType    = "type.googleapis.com/route.v1.RouteConfiguration"
	ClusterType  = "type.googleapis.com/cluster.v1.Cluster"
	EndpointType = "type.googleapis.com/endpoint.v1.ClusterLoadAssignment"
	SecretType   = "type.googleapis.com/extensions.transport_sockets.tls.v1.Secret"
)

type EncodedResource struct {
	Name    string
	Version string
	Value   *anypb.Any
}

type GatewayOutput struct {
	Key       string
	Version   string
	Resources map[string]map[string]EncodedResource
	Statuses  []StatusUpdate
}

func (g GatewayOutput) ResourceName() string            { return g.Key }
func (g GatewayOutput) Equals(other GatewayOutput) bool { return g.Version == other.Version }

func encodeResource(name string, value proto.Message) EncodedResource {
	bytes, err := (proto.MarshalOptions{Deterministic: true}).Marshal(value)
	if err != nil {
		panic(err)
	}
	hash := sha256.Sum256(bytes)
	return EncodedResource{Name: name, Version: hex.EncodeToString(hash[:]), Value: &anypb.Any{
		TypeUrl: "type.googleapis.com/" + string(value.ProtoReflect().Descriptor().FullName()), Value: bytes,
	}}
}

type XDSServer struct {
	discovery.UnimplementedAggregatedDiscoveryServiceServer
	outputs      krt.Collection[GatewayOutput]
	authorize    func(context.Context, string) error
	mu           sync.Mutex
	watchers     map[string]map[chan struct{}]struct{}
	acknowledged map[<-chan struct{}]*streamAcknowledgements
	observer     func(string)
	nonce        atomic.Uint64
	ACKs         atomic.Uint64
	NACKs        atomic.Uint64
	Connections  atomic.Int64
}

type streamAcknowledgements struct {
	key      string
	node     string
	versions map[string]string
}

func NewXDSServer(outputs krt.Collection[GatewayOutput], authorize func(context.Context, string) error) *XDSServer {
	s := &XDSServer{outputs: outputs, authorize: authorize, watchers: map[string]map[chan struct{}]struct{}{}, acknowledged: map[<-chan struct{}]*streamAcknowledgements{}}
	outputs.Register(func(event krt.Event[GatewayOutput]) {
		s.mu.Lock()
		defer s.mu.Unlock()
		for ch := range s.watchers[event.Latest().Key] {
			// Coalesce notifications; each stream reads the newest immutable output.
			select {
			case ch <- struct{}{}:
			default:
			}
		}
	})
	return s
}

func (s *XDSServer) SetObserver(observer func(string)) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.observer = observer
}

func (s *XDSServer) recordAcknowledgement(key string, stream <-chan struct{}, typeURL, version string) {
	s.mu.Lock()
	if current := s.acknowledged[stream]; current != nil {
		current.versions[typeURL] = version
	}
	observer := s.observer
	s.mu.Unlock()
	if observer != nil {
		observer(key)
	}
}

func resourceSetVersion(resources map[string]EncodedResource) string {
	hash := sha256.New()
	for _, name := range slices.Sorted(maps.Keys(resources)) {
		hash.Write([]byte(name + "\x00" + resources[name].Version + "\x00"))
	}
	return hex.EncodeToString(hash.Sum(nil))
}

// An ACK for one resource type cannot mark the whole Gateway as programmed.
func (s *XDSServer) AcknowledgedReplicas(key string) int {
	output := s.outputs.GetKey(key)
	if output == nil || len(output.Resources) == 0 {
		return 0
	}
	expected := map[string]string{}
	for typeURL, resources := range output.Resources {
		expected[typeURL] = resourceSetVersion(resources)
	}
	nodes := map[string]struct{}{}
	s.mu.Lock()
	defer s.mu.Unlock()
	for _, stream := range s.acknowledged {
		if stream.key != key || stream.node == "" {
			continue
		}
		matched := true
		for typeURL, version := range expected {
			if stream.versions[typeURL] != version {
				matched = false
				break
			}
		}
		if matched {
			nodes[stream.node] = struct{}{}
		}
	}
	return len(nodes)
}

func (s *XDSServer) watch(key, node string) (<-chan struct{}, func()) {
	ch := make(chan struct{}, 1)
	s.mu.Lock()
	if s.watchers[key] == nil {
		s.watchers[key] = map[chan struct{}]struct{}{}
	}
	s.watchers[key][ch] = struct{}{}
	s.acknowledged[ch] = &streamAcknowledgements{key: key, node: node, versions: map[string]string{}}
	s.mu.Unlock()
	s.Connections.Add(1)
	return ch, func() {
		s.mu.Lock()
		delete(s.watchers[key], ch)
		delete(s.acknowledged, ch)
		if len(s.watchers[key]) == 0 {
			delete(s.watchers, key)
		}
		observer := s.observer
		s.mu.Unlock()
		s.Connections.Add(-1)
		if observer != nil {
			observer(key)
		}
	}
}

func (s *XDSServer) authenticate(ctx context.Context, node *core.Node) (string, error) {
	key := node.GetMetadata().GetFields()["TRANSIT_GATEWAY"].GetStringValue()
	parts := strings.Split(key, "/")
	if len(parts) != 2 || parts[0] == "" || parts[1] == "" {
		return "", status.Error(codes.InvalidArgument, "TRANSIT_GATEWAY must identify namespace/name")
	}
	if s.authorize == nil {
		return "", status.Error(codes.Unauthenticated, "xDS authentication is not configured")
	}
	if err := s.authorize(ctx, key); err != nil {
		return "", err
	}
	if !s.outputs.HasSynced() {
		return "", status.Error(codes.Unavailable, "Kubernetes resources are still synchronizing")
	}
	if s.outputs.GetKey(key) == nil {
		return "", status.Error(codes.NotFound, "Gateway is not managed by this controller")
	}
	return key, nil
}

func (s *XDSServer) desired(key, typeURL string, includes func(string) bool) (map[string]EncodedResource, string) {
	result := map[string]EncodedResource{}
	output := s.outputs.GetKey(key)
	if output != nil {
		for name, resource := range output.Resources[typeURL] {
			if includes(name) {
				result[name] = resource
			}
		}
	}
	return result, resourceSetVersion(result)
}

func supportedType(value string) bool {
	return slices.Contains([]string{ListenerType, RouteType, ClusterType, EndpointType, SecretType}, value)
}

type deltaSubscription struct {
	wildcard        bool
	names           map[string]bool
	excluded        map[string]bool
	accepted        map[string]string
	pending         map[string]string
	nonce           string
	pendingVersion  string
	rejectedVersion string
	initialized     bool
}

func (d *deltaSubscription) includes(name string) bool {
	return (d.wildcard && !d.excluded[name]) || d.names[name]
}

func (s *XDSServer) DeltaAggregatedResources(stream discovery.AggregatedDiscoveryService_DeltaAggregatedResourcesServer) error {
	first, err := stream.Recv()
	if err != nil {
		return err
	}
	key, err := s.authenticate(stream.Context(), first.GetNode())
	if err != nil {
		return err
	}
	changed, cancel := s.watch(key, first.GetNode().GetId())
	defer cancel()
	subscriptions := map[string]*deltaSubscription{}
	handle := func(request *discovery.DeltaDiscoveryRequest) error {
		if !supportedType(request.TypeUrl) {
			return status.Error(codes.InvalidArgument, "unsupported xDS resource type")
		}
		if request.Node != nil && request.Node.GetMetadata().GetFields()["TRANSIT_GATEWAY"].GetStringValue() != key {
			return status.Error(codes.PermissionDenied, "a stream cannot change its Gateway identity")
		}
		d := subscriptions[request.TypeUrl]
		if d == nil {
			d = &deltaSubscription{wildcard: len(request.ResourceNamesSubscribe) == 0,
				names: map[string]bool{}, excluded: map[string]bool{}, accepted: maps.Clone(request.InitialResourceVersions)}
			if d.accepted == nil {
				d.accepted = map[string]string{}
			}
			subscriptions[request.TypeUrl] = d
		}
		if request.ResponseNonce != "" && request.ResponseNonce == d.nonce {
			if request.ErrorDetail == nil || request.ErrorDetail.Code == 0 {
				d.accepted = d.pending
				s.ACKs.Add(1)
				s.recordAcknowledgement(key, changed, request.TypeUrl, d.pendingVersion)
			} else {
				d.rejectedVersion = d.pendingVersion
				s.NACKs.Add(1)
			}
			d.pending = nil
			d.nonce = ""
		}
		for _, name := range request.ResourceNamesSubscribe {
			if name == "*" {
				d.wildcard = true
			} else {
				d.names[name] = true
				delete(d.excluded, name)
			}
		}
		for _, name := range request.ResourceNamesUnsubscribe {
			if name == "*" {
				d.wildcard = false
			} else {
				delete(d.names, name)
				d.excluded[name] = true
				delete(d.accepted, name)
			}
		}
		if len(request.ResourceNamesSubscribe) > 0 || len(request.ResourceNamesUnsubscribe) > 0 {
			d.initialized = false
			s.recordAcknowledgement(key, changed, request.TypeUrl, "")
		}
		return nil
	}
	send := func() error {
		for _, typeURL := range slices.Sorted(maps.Keys(subscriptions)) {
			d := subscriptions[typeURL]
			if d.pending != nil {
				continue
			}
			desired, version := s.desired(key, typeURL, d.includes)
			if version == d.rejectedVersion {
				continue
			}
			response := &discovery.DeltaDiscoveryResponse{TypeUrl: typeURL, SystemVersionInfo: version}
			versions := map[string]string{}
			for _, name := range slices.Sorted(maps.Keys(desired)) {
				resource := desired[name]
				versions[name] = resource.Version
				if d.accepted[name] != resource.Version {
					response.Resources = append(response.Resources, &discovery.Resource{Name: name, Version: resource.Version, Resource: resource.Value})
				}
			}
			for _, name := range slices.Sorted(maps.Keys(d.accepted)) {
				if _, found := desired[name]; !found {
					response.RemovedResources = append(response.RemovedResources, name)
				}
			}
			if d.initialized && len(response.Resources) == 0 && len(response.RemovedResources) == 0 {
				continue
			}
			response.Nonce = strconv.FormatUint(s.nonce.Add(1), 10)
			if err := stream.Send(response); err != nil {
				return err
			}
			d.initialized = true
			d.pending = versions
			d.pendingVersion = version
			d.nonce = response.Nonce
		}
		return nil
	}
	if err := handle(first); err != nil {
		return err
	}
	if err := send(); err != nil {
		return err
	}
	requests := make(chan *discovery.DeltaDiscoveryRequest, 1)
	errorsCh := make(chan error, 1)
	ctx, stop := context.WithCancel(stream.Context())
	defer stop()
	go func() {
		for {
			request, err := stream.Recv()
			if err != nil {
				errorsCh <- err
				return
			}
			select {
			case requests <- request:
			case <-ctx.Done():
				return
			}
		}
	}()
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-errorsCh:
			if errors.Is(err, io.EOF) {
				return nil
			}
			return err
		case request := <-requests:
			if err := handle(request); err != nil {
				return err
			}
		case <-changed:
		}
		if err := send(); err != nil {
			return err
		}
	}
}

type sotwSubscription struct {
	names           []string
	nonce           string
	pendingVersion  string
	acceptedVersion string
	rejectedVersion string
}

func (s *XDSServer) StreamAggregatedResources(stream discovery.AggregatedDiscoveryService_StreamAggregatedResourcesServer) error {
	first, err := stream.Recv()
	if err != nil {
		return err
	}
	key, err := s.authenticate(stream.Context(), first.GetNode())
	if err != nil {
		return err
	}
	changed, cancel := s.watch(key, first.GetNode().GetId())
	defer cancel()
	subscriptions := map[string]*sotwSubscription{}
	handle := func(request *discovery.DiscoveryRequest) error {
		if !supportedType(request.TypeUrl) {
			return status.Error(codes.InvalidArgument, "unsupported xDS resource type")
		}
		if request.Node != nil && request.Node.GetMetadata().GetFields()["TRANSIT_GATEWAY"].GetStringValue() != key {
			return status.Error(codes.PermissionDenied, "a stream cannot change its Gateway identity")
		}
		d := subscriptions[request.TypeUrl]
		if d == nil {
			d = &sotwSubscription{}
			subscriptions[request.TypeUrl] = d
		}
		if request.ResponseNonce != "" && request.ResponseNonce == d.nonce {
			if request.ErrorDetail == nil || request.ErrorDetail.Code == 0 {
				d.acceptedVersion = d.pendingVersion
				s.ACKs.Add(1)
				s.recordAcknowledgement(key, changed, request.TypeUrl, d.pendingVersion)
			} else {
				d.rejectedVersion = d.pendingVersion
				s.NACKs.Add(1)
			}
			d.nonce = ""
		}
		if !slices.Equal(d.names, request.ResourceNames) {
			d.acceptedVersion = ""
			d.rejectedVersion = ""
			s.recordAcknowledgement(key, changed, request.TypeUrl, "")
		}
		d.names = slices.Clone(request.ResourceNames)
		return nil
	}
	send := func() error {
		for _, typeURL := range slices.Sorted(maps.Keys(subscriptions)) {
			d := subscriptions[typeURL]
			if d.nonce != "" {
				continue
			}
			desired, version := s.desired(key, typeURL, func(name string) bool { return len(d.names) == 0 || slices.Contains(d.names, name) })
			if version == d.acceptedVersion || version == d.rejectedVersion {
				continue
			}
			response := &discovery.DiscoveryResponse{TypeUrl: typeURL, VersionInfo: version, Nonce: strconv.FormatUint(s.nonce.Add(1), 10)}
			for _, name := range slices.Sorted(maps.Keys(desired)) {
				response.Resources = append(response.Resources, desired[name].Value)
			}
			if err := stream.Send(response); err != nil {
				return err
			}
			d.nonce = response.Nonce
			d.pendingVersion = version
		}
		return nil
	}
	if err := handle(first); err != nil {
		return err
	}
	if err := send(); err != nil {
		return err
	}
	requests := make(chan *discovery.DiscoveryRequest, 1)
	errorsCh := make(chan error, 1)
	ctx, stop := context.WithCancel(stream.Context())
	defer stop()
	go func() {
		for {
			request, err := stream.Recv()
			if err != nil {
				errorsCh <- err
				return
			}
			select {
			case requests <- request:
			case <-ctx.Done():
				return
			}
		}
	}()
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-errorsCh:
			if errors.Is(err, io.EOF) {
				return nil
			}
			return err
		case request := <-requests:
			if err := handle(request); err != nil {
				return err
			}
		case <-changed:
		}
		if err := send(); err != nil {
			return err
		}
	}
}
