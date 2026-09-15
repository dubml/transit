package controlplane

import (
	"crypto/tls"
	"fmt"

	core "github.com/dubml/xds-api/core/v1"
	xdstls "github.com/dubml/xds-api/extensions/transport_sockets/tls/v1"
	"google.golang.org/protobuf/types/known/anypb"
	"istio.io/istio/pkg/kube/krt"
	corev1 "k8s.io/api/core/v1"
	gw "sigs.k8s.io/gateway-api/apis/v1"
)

type listenerTLSFailure struct {
	reason  string
	message string
}

func (c Compiler) listenerTLS(ctx krt.HandlerContext, gateway gw.Gateway, listener gw.Listener) (*core.TransportSocket, *xdstls.Secret, *listenerTLSFailure) {
	invalid := func(reason, message string) (*core.TransportSocket, *xdstls.Secret, *listenerTLSFailure) {
		return nil, nil, &listenerTLSFailure{reason: reason, message: message}
	}
	if listener.Protocol == gw.HTTPProtocolType {
		if listener.TLS != nil {
			return invalid("InvalidCertificateRef", "An HTTP listener cannot configure TLS termination")
		}
		return nil, nil, nil
	}
	if listener.TLS == nil || valueOr(listener.TLS.Mode, string(gw.TLSModeTerminate)) != string(gw.TLSModeTerminate) || len(listener.TLS.CertificateRefs) != 1 || len(listener.TLS.Options) != 0 {
		return invalid("InvalidCertificateRef", "HTTPS requires TLS termination with one certificate Secret and no custom TLS options")
	}
	ref := listener.TLS.CertificateRefs[0]
	if valueOr(ref.Group, "") != "" || valueOr(ref.Kind, "Secret") != "Secret" {
		return invalid("InvalidCertificateRef", "Only core Kubernetes Secret certificate references are supported")
	}
	namespace := valueOr(ref.Namespace, gateway.Namespace)
	if !c.referenceAllowed(ctx, "Gateway", gateway.Namespace, "", "Secret", namespace, string(ref.Name)) {
		return invalid("RefNotPermitted", "Cross-namespace certificate reference requires a ReferenceGrant")
	}
	resource := c.Inputs.Get(ctx, "Secret", namespace, string(ref.Name))
	if resource == nil {
		return invalid("InvalidCertificateRef", "The referenced certificate Secret does not exist")
	}
	var secret corev1.Secret
	if resource.Decode(&secret) != nil || secret.Type != corev1.SecretTypeTLS {
		return invalid("InvalidCertificateRef", "The certificate Secret must have type kubernetes.io/tls")
	}
	certificate, key := secret.Data[corev1.TLSCertKey], secret.Data[corev1.TLSPrivateKeyKey]
	if len(certificate) > 1024*1024 || len(key) > 1024*1024 {
		return invalid("InvalidCertificateRef", "The certificate or private key exceeds 1 MiB")
	}
	if _, err := tls.X509KeyPair(certificate, key); err != nil {
		return invalid("InvalidCertificateRef", "The Secret does not contain a valid matching TLS certificate and private key")
	}
	name := fmt.Sprintf("%s/tls/%s/%s", gatewayKey(gateway.Namespace, gateway.Name), namespace, ref.Name)
	context, _ := anypb.New(&xdstls.DownstreamTlsContext{CommonTlsContext: &xdstls.CommonTlsContext{
		TlsCertificateCertificateProviderInstance: &xdstls.CommonTlsContext_CertificateProviderInstance{
			InstanceName: "transit", CertificateName: name,
		},
		AlpnProtocols: []string{"h2", "http/1.1"},
	}})
	return &core.TransportSocket{Name: "transit.transport_sockets.tls", ConfigType: &core.TransportSocket_TypedConfig{TypedConfig: context}},
		&xdstls.Secret{Name: name, Type: &xdstls.Secret_TlsCertificate{TlsCertificate: &xdstls.TlsCertificate{
			CertificateChain: &core.DataSource{Specifier: &core.DataSource_InlineBytes{InlineBytes: certificate}},
			PrivateKey:       &core.DataSource{Specifier: &core.DataSource_InlineBytes{InlineBytes: key}},
		}}}, nil
}
