# Authenticated ADS

Managed gateways connect to `https://dubbod.dubbo-system.svc:26012` and authenticate with their workload certificate. The server certificate must match the endpoint hostname and chain to the mounted CA. The ADS client verifies both; it has no TLS verification bypass.

`GRPC_XDS_BOOTSTRAP` points to the bootstrap mounted by the Inherent injector. Its `certificate_providers.default` must use `file_watcher` with `certificate_file`, `private_key_file`, and `ca_certificate_file`. The ADS client reads these files on every reconnect, so a renewed certificate is used by the next connection. Missing or malformed credentials fail the connection without falling back to plaintext. An explicitly configured HTTP endpoint is available only without a workload bootstrap, for standalone test servers.

The gateway-specific `DXGATE_BOOTSTRAP` still supplies the ADS endpoint, listener subscriptions, and cluster identity. An external endpoint must use HTTPS and a hostname covered by the control-plane certificate. It is independent of the file paths in `GRPC_XDS_BOOTSTRAP`.

ADS transport authentication and SDS serve different purposes. The mounted certificate establishes the ADS connection. Authenticated SDS then delivers `default` and `ROOTCA` for upstream mTLS. Existing ACK/NACK handling and retention of the last accepted configuration remain in place.

## Upgrade

Deploy the matching control-plane and dxgate versions together. Existing anonymous clients cannot connect after control-plane authentication is enabled. Replace old HTTP ADS overrides with HTTPS addresses. Recreate managed Pods whose Secret mount was derived from a ReplicaSet's shared `generateName`; each replacement Pod receives its own credentials Secret. Do not share or copy a sibling Pod's Secret.
