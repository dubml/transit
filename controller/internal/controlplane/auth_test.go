package controlplane

import (
	"context"
	"strings"
	"testing"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	auth "k8s.io/api/authentication/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/client-go/kubernetes/fake"
	ktesting "k8s.io/client-go/testing"
)

func TestTokenAuthorizerGatewayIsolation(t *testing.T) {
	for _, tc := range []struct {
		name, identity, audience string
		authenticated            bool
		code                     codes.Code
	}{
		{"matching Gateway", "system:serviceaccount:team:transit-edge", XDSAudience, true, codes.OK},
		{"another Gateway", "system:serviceaccount:team:transit-other", XDSAudience, true, codes.PermissionDenied},
		{"another namespace", "system:serviceaccount:other:transit-edge", XDSAudience, true, codes.PermissionDenied},
		{"wrong audience", "system:serviceaccount:team:transit-edge", "kubernetes", true, codes.Unauthenticated},
		{"expired token", "system:serviceaccount:team:transit-edge", XDSAudience, false, codes.Unauthenticated},
	} {
		t.Run(tc.name, func(t *testing.T) {
			client := fake.NewClientset()
			client.PrependReactor("create", "tokenreviews", func(action ktesting.Action) (bool, runtime.Object, error) {
				review := action.(ktesting.CreateAction).GetObject().(*auth.TokenReview)
				if review.Spec.Token != "test-token" || len(review.Spec.Audiences) != 1 || review.Spec.Audiences[0] != XDSAudience {
					t.Fatalf("incorrect TokenReview scope: %#v", review.Spec.Audiences)
				}
				return true, &auth.TokenReview{Status: auth.TokenReviewStatus{Authenticated: tc.authenticated, Audiences: []string{tc.audience}, User: auth.UserInfo{Username: tc.identity}}}, nil
			})
			ctx := metadata.NewIncomingContext(context.Background(), metadata.Pairs("authorization", "Bearer test-token"))
			if got := status.Code(TokenAuthorizer(client.AuthenticationV1())(ctx, "team/edge")); got != tc.code {
				t.Fatalf("got %s, want %s", got, tc.code)
			}
		})
	}
}

func TestTokenAuthorizerRejectsMissingOrAmbiguousCredentials(t *testing.T) {
	client := fake.NewClientset()
	for _, md := range []metadata.MD{nil, metadata.Pairs("authorization", "Bearer "), metadata.Pairs("authorization", "Bearer a", "authorization", "Bearer b")} {
		err := TokenAuthorizer(client.AuthenticationV1())(metadata.NewIncomingContext(context.Background(), md), "team/edge")
		if status.Code(err) != codes.Unauthenticated {
			t.Fatalf("expected rejection, got %v", err)
		}
	}
	if len(client.Actions()) != 0 {
		t.Fatal("malformed credentials must not reach the API server")
	}
}

func TestDataPlaneNamesStayDistinctAndWithinDNSLimit(t *testing.T) {
	a, b := DataPlaneName(strings.Repeat("a", 70)+"1"), DataPlaneName(strings.Repeat("a", 70)+"2")
	if len(a) > 63 || len(b) > 63 || a == b {
		t.Fatalf("invalid generated names: %s, %s", a, b)
	}
}
