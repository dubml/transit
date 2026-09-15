package controlplane

import (
	"context"
	"crypto/sha256"
	"fmt"
	"slices"
	"strings"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	auth "k8s.io/api/authentication/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	authclient "k8s.io/client-go/kubernetes/typed/authentication/v1"
)

const XDSAudience = "transit-xds"

// Keep long Gateway names legal for Service and ServiceAccount names without collisions.
func DataPlaneName(gateway string) string {
	name := "transit-" + strings.ReplaceAll(gateway, ".", "-")
	if len(name) <= 63 && !strings.Contains(gateway, ".") {
		return name
	}
	digest := sha256.Sum256([]byte(gateway))
	return fmt.Sprintf("%s-%x", strings.TrimRight(name[:min(len(name), 46)], "-"), digest[:8])
}

func TokenAuthorizer(client authclient.AuthenticationV1Interface) func(context.Context, string) error {
	return func(ctx context.Context, gateway string) error {
		parts := strings.Split(gateway, "/")
		if len(parts) != 2 || parts[0] == "" || parts[1] == "" {
			return status.Error(codes.Unauthenticated, "invalid Gateway identity")
		}
		md, _ := metadata.FromIncomingContext(ctx)
		values := md.Get("authorization")
		if len(values) != 1 || !strings.HasPrefix(values[0], "Bearer ") {
			return status.Error(codes.Unauthenticated, "xDS requires a ServiceAccount bearer token")
		}
		token := strings.TrimPrefix(values[0], "Bearer ")
		if len(token) == 0 || len(token) > 32768 {
			return status.Error(codes.Unauthenticated, "invalid xDS token length")
		}
		ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
		defer cancel()
		review, err := client.TokenReviews().Create(ctx, &auth.TokenReview{Spec: auth.TokenReviewSpec{
			Token: token, Audiences: []string{XDSAudience},
		}}, metav1.CreateOptions{})
		if err != nil {
			return status.Error(codes.Unavailable, "ServiceAccount token verification is unavailable")
		}
		if !review.Status.Authenticated || !slices.Contains(review.Status.Audiences, XDSAudience) {
			return status.Error(codes.Unauthenticated, "xDS token verification failed")
		}
		expected := "system:serviceaccount:" + parts[0] + ":" + DataPlaneName(parts[1])
		if review.Status.User.Username != expected {
			return status.Error(codes.PermissionDenied, "ServiceAccount does not belong to the requested Gateway")
		}
		return nil
	}
}
