# Transit Kubernetes controller

`transit-controller` watches Gateway API and `TransitService` resources, derives configuration with krt, and serves authenticated TLS xDS. Each managed Gateway gets its own Rust Transit Deployment, Service, ServiceAccount and credential access rules. The data plane receives `--mode=kubernetes`; standalone Transit runs separately with local configuration.

The Rust proxy follows configured HTTP/HTTPS listener ports, withdraws removed sockets and reloads serving certificates from SDS for new TLS connections. Controller lifecycle and proxy traffic are tested with a real API Server and etcd, and with actual ARM64 pods and external NodePort requests in Kubernetes. Verification details and supported boundaries are recorded in [the implementation audit](../docs/runtime-modes-audit.md).

## Build

Run from the Transit repository root:

```sh
docker build -f controller/Dockerfile -t ghcr.io/dubml/transit-controller:dev .
docker build -t ghcr.io/dubml/transit:dev .
```

The `dev` tags identify locally built images; this repository does not establish that they are published. Make them available to the target cluster, or edit the controller image and `--proxy-image` in `install/control-plane.yaml` to use your registry.

## Install

Use a Kubernetes context selected for this installation. Install the Gateway API definitions matching the controller dependency, then create the controller namespace:

```sh
gateway_api_dir="$(cd controller && go list -m -f '{{.Dir}}' sigs.k8s.io/gateway-api)"
kubectl apply --server-side -f "$gateway_api_dir/config/crd/standard"
kubectl apply -f controller/install/namespace.yaml
```

Create the xDS certificate Secret from `tls.crt`, `tls.key` and its trust bundle `ca.crt`. The certificate must cover `transit-control-plane.transit-system.svc`. The controller serves the certificate; each generated data plane receives the trust bundle and a projected ServiceAccount token with audience `transit-xds`.

```sh
kubectl -n transit-system create secret generic transit-xds-tls \
  --from-file=tls.crt --from-file=tls.key --from-file=ca.crt
kubectl apply -k controller/install
kubectl -n transit-system rollout status deployment/transit-controller
```

The controller currently runs as one replica with a Recreate rollout. A managed Gateway defaults to one proxy replica and a LoadBalancer Service. Set `--proxy-replicas` or `--service-type` on the controller when needed. A cluster without a LoadBalancer provisioner can use `ClusterIP` and Service port forwarding for local verification.

The controller reloads its serving certificate for new TLS connections. Changes to the xDS CA trust bundle require a controller restart to redistribute the bundle to existing data plane ConfigMaps; automatic CA rotation and controller high availability are not implemented. A missing Gateway serving certificate withdraws HTTPS from the proxy while retaining declared Service ports and allocated NodePorts. Restoring the Secret restores the listener at the same external address. Removing the Gateway Listener explicitly removes its Service port.

Set Gateway HTTP listeners to port `26080` and HTTPS listeners to `26443`. Each becomes a container port and matching Service port; HTTPS also needs a valid certificate Secret. Listener ports are explicit Gateway settings and custom ports are preserved. Data plane health probes use the named management port `26021`; it is excluded from the API Service. For local API access, forward the Service's HTTP port with `kubectl -n <namespace> port-forward service/<data-plane-service> 26080:26080`, or its HTTPS port with `26443:26443`. Production access uses the Service's LoadBalancer address or allocated NodePort.

Ordinary HTTPRoute backends reference Kubernetes Services. LLM, MCP and A2A backends reference `networking.dubbo.apache.org/v1alpha3` `TransitService`; its bundled CRD comes from `dubml/api`.

## Verify

```sh
cd controller
GOWORK=off go test ./...
GOWORK=off go test -tags=integration ./... -count=1
```

The integration test downloads Kubernetes 1.36.0 envtest binaries into the temporary directory `transit-mode-envtest`, or uses `KUBEBUILDER_ASSETS` when supplied. It starts its own API Server and etcd and does not use the current Kubernetes context. It installs the bundled RBAC, runs the controller as that ServiceAccount, exercises TLS xDS with a real TokenReview, checks deployment updates and verifies status transitions and resource ownership. Deployment readiness is supplied by the test because envtest has no scheduler or kubelet.

To additionally launch the real Rust proxy and verify HTTP/HTTPS traffic for HTTP, LLM, MCP and A2A, run from the repository root:

```sh
cargo build -p transit-app --bin transit
cd controller
TRANSIT_PROXY_BINARY="$(cd ../target/debug && pwd)/transit" GOWORK=off go test -tags=integration ./... -count=1
```

With a custom Cargo target directory, supply its absolute binary path in `TRANSIT_PROXY_BINARY`. Without that variable, the proxy traffic subtest explicitly skips. Upstreams are local test servers; the test does not contact model providers or prove cluster scheduling. It also checks certificate rotation and HTTPS withdrawal while HTTP remains available.

HTTPS currently supports termination using one `kubernetes.io/tls` Secret per listener, with a ReferenceGrant for cross-namespace certificate references. Shared sockets must use the same certificate; distinct SNI certificates and downstream client-certificate authentication are not implemented. LLM credential references have a separate namespace boundary, tracked in the audit.

`/healthz` checks the controller process. `/readyz` becomes ready after both input and managed-resource caches synchronize. Gateway `Programmed=True` additionally requires the desired Deployment generation to be available, a Service address and current xDS acknowledgements for all published resource types from enough distinct proxy nodes.
