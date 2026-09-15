package controlplane

import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"math/big"
	"testing"
	"time"

	xdstls "github.com/dubml/xds-api/extensions/transport_sockets/tls/v1"
	listener "github.com/dubml/xds-api/listener/v1"
	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime"
)

func testServingCertificate(t *testing.T) ([]byte, []byte) {
	t.Helper()
	public, private, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	cert := &x509.Certificate{SerialNumber: big.NewInt(1), Subject: pkix.Name{CommonName: "localhost"}, DNSNames: []string{"localhost"}, NotBefore: time.Now().Add(-time.Minute), NotAfter: time.Now().Add(time.Hour), KeyUsage: x509.KeyUsageDigitalSignature, ExtKeyUsage: []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth}}
	der, err := x509.CreateCertificate(rand.Reader, cert, cert, public, private)
	if err != nil {
		t.Fatal(err)
	}
	key, err := x509.MarshalPKCS8PrivateKey(private)
	if err != nil {
		t.Fatal(err)
	}
	return pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: der}), pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: key})
}

func inputServingCertificate(t *testing.T, inputs Inputs, certificate, key []byte) {
	t.Helper()
	object, err := runtime.DefaultUnstructuredConverter.ToUnstructured(&corev1.Secret{
		ObjectMeta: metav1.ObjectMeta{Namespace: "certificates", Name: "serving"}, Type: corev1.SecretTypeTLS,
		Data: map[string][]byte{corev1.TLSCertKey: certificate, corev1.TLSPrivateKeyKey: key},
	})
	if err != nil {
		t.Fatal(err)
	}
	inputs.Objects.ConditionalUpdateObject(NewResource("Secret", &unstructured.Unstructured{Object: object}))
}

func TestHTTPSTerminationTracksCertificateAndReferenceGrant(t *testing.T) {
	inputs, outputs := testGateway(t, `{"ai":{"provider":{"openai":{}}}}`)
	certificate, key := testServingCertificate(t)
	inputServingCertificate(t, inputs, certificate, key)
	inputJSON(t, inputs, "Gateway", "team", "edge", `{"gatewayClassName":"transit","listeners":[{"name":"https","port":8443,"protocol":"HTTPS","tls":{"mode":"Terminate","certificateRefs":[{"name":"serving","namespace":"certificates"}]}}]}`)
	awaitReason := func(reason string) {
		t.Helper()
		awaitGateway(t, outputs, func(out *GatewayOutput) bool {
			for _, status := range out.Statuses {
				for _, listener := range status.Listeners {
					for _, condition := range listener.Conditions {
						if condition.Type == "ResolvedRefs" && condition.Reason == reason && condition.Status == metav1.ConditionFalse {
							return len(out.Resources[ListenerType]) == 0 && len(out.Resources[SecretType]) == 0
						}
					}
				}
			}
			return false
		})
	}
	awaitReason("RefNotPermitted")
	grant := `{"from":[{"group":"gateway.networking.k8s.io","kind":"Gateway","namespace":"team"}],"to":[{"group":"","kind":"Secret","name":"serving"}]}`
	inputJSON(t, inputs, "ReferenceGrant", "certificates", "allow", grant)
	out := awaitGateway(t, outputs, func(out *GatewayOutput) bool { return len(out.Resources[SecretType]) == 1 })
	lds := &listener.Listener{}
	if err := out.Resources[ListenerType]["team/edge/https"].Value.UnmarshalTo(lds); err != nil {
		t.Fatal(err)
	}
	if lds.ApiListener != nil || len(lds.FilterChains) != 1 {
		t.Fatal("HTTPS must use a network filter chain")
	}
	context := &xdstls.DownstreamTlsContext{}
	if err := lds.FilterChains[0].TransportSocket.GetTypedConfig().UnmarshalTo(context); err != nil {
		t.Fatal(err)
	}
	name := context.CommonTlsContext.TlsCertificateCertificateProviderInstance.CertificateName
	secret := &xdstls.Secret{}
	if err := out.Resources[SecretType][name].Value.UnmarshalTo(secret); err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(secret.GetTlsCertificate().CertificateChain.GetInlineBytes(), certificate) {
		t.Fatal("wrong certificate published")
	}
	oldVersion := out.Resources[SecretType][name].Version
	certificate, key = testServingCertificate(t)
	inputServingCertificate(t, inputs, certificate, key)
	awaitGateway(t, outputs, func(out *GatewayOutput) bool {
		value, ok := out.Resources[SecretType][name]
		return ok && value.Version != oldVersion
	})
	inputs.Objects.DeleteObject(resourceKey("ReferenceGrant", "certificates", "allow"))
	awaitReason("RefNotPermitted")
	inputJSON(t, inputs, "ReferenceGrant", "certificates", "allow", grant)
	inputServingCertificate(t, inputs, certificate, []byte("invalid-key"))
	awaitReason("InvalidCertificateRef")
	inputServingCertificate(t, inputs, certificate, key)
	awaitGateway(t, outputs, func(out *GatewayOutput) bool { return len(out.Resources[SecretType]) == 1 })
	inputs.Objects.DeleteObject(resourceKey("Secret", "certificates", "serving"))
	awaitReason("InvalidCertificateRef")
}
