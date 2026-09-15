package controlplane

import (
	"context"
	"net"
	"testing"
	"time"

	core "github.com/dubml/xds-api/core/v1"
	listener "github.com/dubml/xds-api/listener/v1"
	route "github.com/dubml/xds-api/route/v1"
	discovery "github.com/dubml/xds-api/service/discovery/v1"
	rpc "google.golang.org/genproto/googleapis/rpc/status"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
	"google.golang.org/protobuf/types/known/structpb"
	"istio.io/istio/pkg/kube/krt"
)

func TestAcknowledgedReplicasRequireCurrentCompleteConfigAndDistinctNodes(t *testing.T) {
	outputs := krt.NewStaticCollection[GatewayOutput](nil, nil, krt.WithStop(t.Context().Done()))
	output := xdsTestOutput(8080)
	rds := encodeResource("route", &route.RouteConfiguration{Name: "route"})
	output.Resources[RouteType] = map[string]EncodedResource{"route": rds}
	output.finishVersion()
	outputs.UpdateObject(output)
	server := NewXDSServer(outputs, nil)
	stream, closeStream := server.watch(output.Key, "pod-a")
	server.recordAcknowledgement(output.Key, stream, ListenerType, resourceSetVersion(output.Resources[ListenerType]))
	if server.AcknowledgedReplicas(output.Key) != 0 {
		t.Fatal("partial ACK marked a replica programmed")
	}
	server.recordAcknowledgement(output.Key, stream, RouteType, resourceSetVersion(output.Resources[RouteType]))
	if server.AcknowledgedReplicas(output.Key) != 1 {
		t.Fatal("complete ACK was not counted")
	}
	duplicate, closeDuplicate := server.watch(output.Key, "pod-a")
	defer closeDuplicate()
	for kind, resources := range output.Resources {
		server.recordAcknowledgement(output.Key, duplicate, kind, resourceSetVersion(resources))
	}
	if server.AcknowledgedReplicas(output.Key) != 1 {
		t.Fatal("two streams from the same node counted twice")
	}
	closeStream()
	if server.AcknowledgedReplicas(output.Key) != 1 {
		t.Fatal("remaining stream lost its acknowledgement")
	}
	changed := xdsTestOutput(9090)
	changed.Resources[RouteType] = output.Resources[RouteType]
	changed.finishVersion()
	outputs.UpdateObject(changed)
	if server.AcknowledgedReplicas(output.Key) != 0 {
		t.Fatal("stale ACK survived a configuration change")
	}
	server.recordAcknowledgement(output.Key, duplicate, ListenerType, resourceSetVersion(changed.Resources[ListenerType]))
	if server.AcknowledgedReplicas(output.Key) != 1 {
		t.Fatal("fresh listener ACK was not counted")
	}
}

func xdsTestClient(t *testing.T) (discovery.AggregatedDiscoveryServiceClient, krt.StaticCollection[GatewayOutput], context.Context) {
	t.Helper()
	ctx, cancel := context.WithTimeout(t.Context(), 5*time.Second)
	t.Cleanup(cancel)
	outputs := krt.NewStaticCollection[GatewayOutput](nil, nil, krt.WithStop(ctx.Done()))
	outputs.UpdateObject(GatewayOutput{Key: "team/edge", Version: "empty", Resources: map[string]map[string]EncodedResource{}})
	ads := NewXDSServer(outputs, func(_ context.Context, key string) error {
		if key != "team/edge" {
			t.Errorf("unexpected Gateway: %s", key)
		}
		return nil
	})
	transport := bufconn.Listen(1024 * 1024)
	server := grpc.NewServer()
	discovery.RegisterAggregatedDiscoveryServiceServer(server, ads)
	go func() { _ = server.Serve(transport) }()
	t.Cleanup(func() { server.Stop(); _ = transport.Close() })
	conn, err := grpc.NewClient("passthrough:///xds-test", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithContextDialer(func(ctx context.Context, _ string) (net.Conn, error) { return transport.DialContext(ctx) }))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = conn.Close() })
	return discovery.NewAggregatedDiscoveryServiceClient(conn), outputs, ctx
}

func xdsTestNode() *core.Node {
	return &core.Node{Id: "proxy", Metadata: &structpb.Struct{Fields: map[string]*structpb.Value{"TRANSIT_GATEWAY": structpb.NewStringValue("team/edge")}}}
}

func xdsTestOutput(port uint32) GatewayOutput {
	value := &listener.Listener{Name: "http", Address: &core.Address{Address: &core.Address_SocketAddress{SocketAddress: &core.SocketAddress{Address: "0.0.0.0", PortSpecifier: &core.SocketAddress_PortValue{PortValue: port}}}}}
	resource := encodeResource("http", value)
	return GatewayOutput{Key: "team/edge", Version: resource.Version, Resources: map[string]map[string]EncodedResource{ListenerType: {"http": resource}}}
}

func TestDeltaXDSUpdateNACKAndDeletion(t *testing.T) {
	client, outputs, ctx := xdsTestClient(t)
	outputs.UpdateObject(xdsTestOutput(8080))
	stream, err := client.DeltaAggregatedResources(ctx)
	if err != nil {
		t.Fatal(err)
	}
	send := func(request *discovery.DeltaDiscoveryRequest) {
		t.Helper()
		if err := stream.Send(request); err != nil {
			t.Fatal(err)
		}
	}
	recv := func() *discovery.DeltaDiscoveryResponse {
		t.Helper()
		response, err := stream.Recv()
		if err != nil {
			t.Fatal(err)
		}
		return response
	}
	send(&discovery.DeltaDiscoveryRequest{Node: xdsTestNode(), TypeUrl: ListenerType})
	initial := recv()
	if len(initial.Resources) != 1 {
		t.Fatalf("initial resources: %v", initial)
	}
	send(&discovery.DeltaDiscoveryRequest{TypeUrl: ListenerType, ResponseNonce: initial.Nonce})
	outputs.UpdateObject(xdsTestOutput(8081))
	rejected := recv()
	if len(rejected.Resources) != 1 || rejected.Resources[0].Version == initial.Resources[0].Version {
		t.Fatal("changed listener was not sent")
	}
	send(&discovery.DeltaDiscoveryRequest{TypeUrl: ListenerType, ResponseNonce: rejected.Nonce, ErrorDetail: &rpc.Status{Code: int32(codes.InvalidArgument), Message: "test rejection"}})
	outputs.UpdateObject(xdsTestOutput(8082))
	recovered := recv()
	if len(recovered.Resources) != 1 || recovered.Resources[0].Version != xdsTestOutput(8082).Resources[ListenerType]["http"].Version {
		t.Fatal("NACK prevented a corrected resource from being sent")
	}
	send(&discovery.DeltaDiscoveryRequest{TypeUrl: ListenerType, ResponseNonce: recovered.Nonce})
	outputs.UpdateObject(GatewayOutput{Key: "team/edge", Version: "deleted", Resources: map[string]map[string]EncodedResource{}})
	deleted := recv()
	if len(deleted.Resources) != 0 || len(deleted.RemovedResources) != 1 || deleted.RemovedResources[0] != "http" {
		t.Fatalf("missing explicit deletion: %v", deleted)
	}
}

func TestDeltaXDSReconnectRemovesStaleClientResources(t *testing.T) {
	client, _, ctx := xdsTestClient(t)
	stream, err := client.DeltaAggregatedResources(ctx)
	if err != nil {
		t.Fatal(err)
	}
	err = stream.Send(&discovery.DeltaDiscoveryRequest{Node: xdsTestNode(), TypeUrl: ListenerType, InitialResourceVersions: map[string]string{"stale": "old-version"}})
	if err != nil {
		t.Fatal(err)
	}
	response, err := stream.Recv()
	if err != nil {
		t.Fatal(err)
	}
	if len(response.RemovedResources) != 1 || response.RemovedResources[0] != "stale" {
		t.Fatalf("stale resource survived reconnect: %v", response)
	}
}

func TestSotWXDSSubscriptionsAndDeletion(t *testing.T) {
	client, outputs, ctx := xdsTestClient(t)
	outputs.UpdateObject(xdsTestOutput(8080))
	stream, err := client.StreamAggregatedResources(ctx)
	if err != nil {
		t.Fatal(err)
	}
	if err := stream.Send(&discovery.DiscoveryRequest{Node: xdsTestNode(), TypeUrl: ListenerType, ResourceNames: []string{"http"}}); err != nil {
		t.Fatal(err)
	}
	initial, err := stream.Recv()
	if err != nil {
		t.Fatal(err)
	}
	if len(initial.Resources) != 1 {
		t.Fatalf("missing subscribed resource: %v", initial)
	}
	if err := stream.Send(&discovery.DiscoveryRequest{TypeUrl: ListenerType, ResourceNames: []string{"http"}, ResponseNonce: initial.Nonce, VersionInfo: initial.VersionInfo}); err != nil {
		t.Fatal(err)
	}
	outputs.UpdateObject(GatewayOutput{Key: "team/edge", Version: "deleted", Resources: map[string]map[string]EncodedResource{}})
	deleted, err := stream.Recv()
	if err != nil {
		t.Fatal(err)
	}
	if len(deleted.Resources) != 0 || deleted.VersionInfo == initial.VersionInfo {
		t.Fatalf("SotW deletion not reflected: %v", deleted)
	}
}
