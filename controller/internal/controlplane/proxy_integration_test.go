//go:build integration

package controlplane

import (
	"bytes"
	"context"
	"crypto/tls"
	"crypto/x509"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"testing"
	"time"

	authentication "k8s.io/api/authentication/v1"
	core "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/client-go/dynamic"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/util/retry"
	gw "sigs.k8s.io/gateway-api/apis/v1"
	gwclient "sigs.k8s.io/gateway-api/pkg/client/clientset/versioned"
)

func testRustProxyTraffic(t *testing.T, ctx context.Context, kube kubernetes.Interface, gateways gwclient.Interface, dynamicClient dynamic.Interface, xdsAddress, caFile string) {
	binary := os.Getenv("TRANSIT_PROXY_BINARY")
	if binary == "" {
		t.Skip("set TRANSIT_PROXY_BINARY to a built transit binary for Rust/controller network integration")
	}
	if !filepath.IsAbs(binary) {
		t.Fatal("TRANSIT_PROXY_BINARY must be an absolute path")
	}
	upstream := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		switch r.URL.Path {
		case "/v1/chat/completions":
			io.WriteString(w, `{"id":"completion","object":"chat.completion","model":"test-model","choices":[{"index":0,"message":{"role":"assistant","content":"llm-ok"},"finish_reason":"stop"}],"usage":{"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}}`)
		case "/mcp":
			io.WriteString(w, `{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{},"serverInfo":{"name":"mcp-ok","version":"1"}}}`)
		case "/agent":
			io.WriteString(w, `{"jsonrpc":"2.0","id":1,"result":{"id":"a2a-ok","contextId":"context","status":{"state":"completed"},"kind":"task"}}`)
		default:
			io.WriteString(w, `{"result":"http-ok"}`)
		}
	}))
	t.Cleanup(upstream.Close)
	_, upstreamPortString, _ := net.SplitHostPort(strings.TrimPrefix(upstream.URL, "http://"))
	upstreamPort, _ := strconv.Atoi(upstreamPortString)
	_, err := kube.CoreV1().Services("team").Create(ctx, &core.Service{ObjectMeta: metav1.ObjectMeta{Name: "rust-upstream"}, Spec: core.ServiceSpec{
		Type: core.ServiceTypeExternalName, ExternalName: "localhost", Ports: []core.ServicePort{{Port: int32(upstreamPort)}},
	}}, metav1.CreateOptions{})
	if err != nil {
		t.Fatal(err)
	}
	certificate, privateKey := testServingCertificate(t)
	_, err = kube.CoreV1().Secrets("team").Create(ctx, &core.Secret{ObjectMeta: metav1.ObjectMeta{Name: "rust-serving"}, Type: core.SecretTypeTLS, Data: map[string][]byte{core.TLSCertKey: certificate, core.TLSPrivateKeyKey: privateKey}}, metav1.CreateOptions{})
	if err != nil {
		t.Fatal(err)
	}
	port := func() (string, gw.PortNumber) {
		address := testListenAddress(t)
		_, value, _ := net.SplitHostPort(address)
		parsed, _ := strconv.Atoi(value)
		return address, gw.PortNumber(parsed)
	}
	httpAddress, httpPort := port()
	httpsAddress, httpsPort := port()
	uiAddress := testListenAddress(t)
	gateway, err := gateways.GatewayV1().Gateways("team").Create(ctx, &gw.Gateway{ObjectMeta: metav1.ObjectMeta{Name: "rust"}, Spec: gw.GatewaySpec{GatewayClassName: "transit", Listeners: []gw.Listener{
		{Name: "http", Protocol: gw.HTTPProtocolType, Port: httpPort},
		{Name: "https", Protocol: gw.HTTPSProtocolType, Port: httpsPort, TLS: &gw.ListenerTLSConfig{CertificateRefs: []gw.SecretObjectReference{{Name: "rust-serving"}}}},
	}}}, metav1.CreateOptions{})
	if err != nil {
		t.Fatal(err)
	}
	serviceType, _ := resourceType("TransitService")
	for name, spec := range map[string]string{
		"rust-llm": fmt.Sprintf(`{"ai":{"provider":{"openai":{}},"endpoint":%q}}`, upstream.URL+"/v1"),
		"rust-mcp": fmt.Sprintf(`{"mcp":{"targets":[{"name":"tools","static":{"backendRef":{"name":"rust-upstream"},"port":%d,"path":"/mcp"}}]}}`, upstreamPort),
		"rust-a2a": fmt.Sprintf(`{"a2a":{"backendRef":{"name":"rust-upstream"},"port":%d,"path":"/agent"}}`, upstreamPort),
	} {
		var object map[string]any
		if err := json.Unmarshal([]byte(fmt.Sprintf(`{"apiVersion":%q,"kind":"TransitService","metadata":{"name":%q},"spec":%s}`, TransitGroup+"/"+TransitVersion, name, spec)), &object); err != nil {
			t.Fatal(err)
		}
		if _, err := dynamicClient.Resource(serviceType.GVR).Namespace("team").Create(ctx, &unstructured.Unstructured{Object: object}, metav1.CreateOptions{}); err != nil {
			t.Fatal(err)
		}
	}
	for name, path := range map[string]string{"rust-http": "/http", "rust-llm": "/v1", "rust-mcp": "/mcp", "rust-a2a": "/agent"} {
		group, kind := gw.Group(TransitGroup), gw.Kind("TransitService")
		backend := gw.BackendObjectReference{Group: &group, Kind: &kind, Name: gw.ObjectName(name)}
		if name == "rust-http" {
			value := gw.PortNumber(upstreamPort)
			backend = gw.BackendObjectReference{Name: "rust-upstream", Port: &value}
		}
		pathType := gw.PathMatchPathPrefix
		_, err := gateways.GatewayV1().HTTPRoutes("team").Create(ctx, &gw.HTTPRoute{ObjectMeta: metav1.ObjectMeta{Name: name}, Spec: gw.HTTPRouteSpec{
			CommonRouteSpec: gw.CommonRouteSpec{ParentRefs: []gw.ParentReference{{Name: "rust"}}},
			Rules:           []gw.HTTPRouteRule{{Matches: []gw.HTTPRouteMatch{{Path: &gw.HTTPPathMatch{Type: &pathType, Value: &path}}}, BackendRefs: []gw.HTTPBackendRef{{BackendRef: gw.BackendRef{BackendObjectReference: backend}}}}},
		}}, metav1.CreateOptions{})
		if err != nil {
			t.Fatal(err)
		}
	}
	name := DataPlaneName(gateway.Name)
	waitFor(t, "Rust proxy ServiceAccount", func() bool {
		_, err := kube.CoreV1().ServiceAccounts("team").Get(ctx, name, metav1.GetOptions{})
		return err == nil
	})
	token, err := kube.CoreV1().ServiceAccounts("team").CreateToken(ctx, name, &authentication.TokenRequest{Spec: authentication.TokenRequestSpec{Audiences: []string{XDSAudience}}}, metav1.CreateOptions{})
	if err != nil {
		t.Fatal(err)
	}
	tokenFile := filepath.Join(t.TempDir(), "xds-token")
	if err := os.WriteFile(tokenFile, []byte(token.Status.Token), 0600); err != nil {
		t.Fatal(err)
	}
	logDirectory := os.Getenv("TRANSIT_E2E_EVIDENCE_DIR")
	if logDirectory == "" {
		logDirectory = t.TempDir()
	}
	logFile, err := os.Create(filepath.Join(logDirectory, "rust-proxy-integration.log"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { logFile.Close() })
	command := exec.Command(binary, "--mode", "kubernetes", "--gateway", "team/rust", "--namespace", "team", "--pod-name", "rust-integration", "--pod-ip", "127.0.0.1", "--xds-address", "https://"+xdsAddress, "--xds-root-ca", caFile, "--xds-token-file", tokenFile, "--ui-addr", uiAddress, "--drain-timeout-seconds", "1")
	for _, variable := range os.Environ() {
		if !strings.HasPrefix(variable, "TRANSIT_") && !strings.HasPrefix(variable, "OTEL_") {
			command.Env = append(command.Env, variable)
		}
	}
	command.Stdout, command.Stderr = logFile, logFile
	if err := command.Start(); err != nil {
		t.Fatal(err)
	}
	finished := make(chan error, 1)
	go func() { finished <- command.Wait() }()
	t.Cleanup(func() {
		_ = command.Process.Signal(os.Interrupt)
		select {
		case err := <-finished:
			if err != nil {
				t.Errorf("Rust proxy failed: %v; log: %s", err, logFile.Name())
			}
		case <-time.After(5 * time.Second):
			_ = command.Process.Kill()
			<-finished
			t.Error("Rust proxy did not stop within the drain deadline")
		}
	})
	client := &http.Client{Timeout: 2 * time.Second}
	waitFor(t, "Rust proxy readiness", func() bool {
		response, err := client.Get("http://" + uiAddress + "/readyz")
		if err != nil {
			return false
		}
		defer response.Body.Close()
		return response.StatusCode == 200
	})
	roots := x509.NewCertPool()
	roots.AppendCertsFromPEM(certificate)
	transport := &http.Transport{TLSClientConfig: &tls.Config{MinVersion: tls.VersionTLS12, RootCAs: roots, ServerName: "localhost"}, DisableKeepAlives: true}
	t.Cleanup(transport.CloseIdleConnections)
	tlsClient := &http.Client{Timeout: 2 * time.Second, Transport: transport}
	check := func(client *http.Client, base, path, body, marker string) {
		t.Helper()
		lastResponse := "no response"
		defer func() {
			if t.Failed() {
				t.Logf("last %s response: %s", marker, lastResponse)
			}
		}()
		waitFor(t, "proxied "+marker+" via "+base, func() bool {
			method := "GET"
			if body != "" {
				method = "POST"
			}
			request, err := http.NewRequestWithContext(ctx, method, base+path, strings.NewReader(body))
			if err != nil {
				return false
			}
			request.Header.Set("Content-Type", "application/json")
			response, err := client.Do(request)
			if err != nil {
				lastResponse = err.Error()
				return false
			}
			defer response.Body.Close()
			data, _ := io.ReadAll(response.Body)
			lastResponse = fmt.Sprintf("status=%d body=%s", response.StatusCode, data)
			return response.StatusCode == 200 && bytes.Contains(data, []byte(marker))
		})
	}
	for _, endpoint := range []struct {
		client *http.Client
		url    string
	}{{client, "http://" + httpAddress}, {tlsClient, "https://" + httpsAddress}} {
		check(endpoint.client, endpoint.url, "/http", "", "http-ok")
		check(endpoint.client, endpoint.url, "/v1/chat/completions", `{"model":"test-model","messages":[{"role":"user","content":"hi"}]}`, "llm-ok")
		check(endpoint.client, endpoint.url, "/mcp", `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}`, "mcp-ok")
		check(endpoint.client, endpoint.url, "/agent", `{"jsonrpc":"2.0","id":1,"method":"message/send","params":{"message":{"role":"user","parts":[{"kind":"text","text":"hi"}]}}}`, "a2a-ok")
	}
	certificate, privateKey = testServingCertificate(t)
	if err := retry.RetryOnConflict(retry.DefaultRetry, func() error {
		secret, err := kube.CoreV1().Secrets("team").Get(ctx, "rust-serving", metav1.GetOptions{})
		if err != nil {
			return err
		}
		secret.Data = map[string][]byte{core.TLSCertKey: certificate, core.TLSPrivateKeyKey: privateKey}
		_, err = kube.CoreV1().Secrets("team").Update(ctx, secret, metav1.UpdateOptions{})
		return err
	}); err != nil {
		t.Fatal(err)
	}
	newRoots := x509.NewCertPool()
	newRoots.AppendCertsFromPEM(certificate)
	rotatedTransport := &http.Transport{TLSClientConfig: &tls.Config{MinVersion: tls.VersionTLS12, RootCAs: newRoots, ServerName: "localhost"}, DisableKeepAlives: true}
	t.Cleanup(rotatedTransport.CloseIdleConnections)
	check(&http.Client{Timeout: 2 * time.Second, Transport: rotatedTransport}, "https://"+httpsAddress, "/http", "", "http-ok")
	if err := kube.CoreV1().Secrets("team").Delete(ctx, "rust-serving", metav1.DeleteOptions{}); err != nil {
		t.Fatal(err)
	}
	waitFor(t, "withdrawn HTTPS socket", func() bool {
		connection, err := net.DialTimeout("tcp", httpsAddress, time.Second)
		if err != nil {
			return true
		}
		connection.Close()
		return false
	})
	check(client, "http://"+httpAddress, "/http", "", "http-ok")
}
