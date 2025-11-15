package interceptor

import (
	"context"
	"fmt"
	"strings"

	"brume.dev/internal/common"
	"brume.dev/internal/log"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

var logger = log.GetLogger("router.grpc.interceptor")

// authentificationInterceptor is a gRPC interceptor that checks if the user is authenticated
func AuthentificationInterceptor(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (resp any, err error) {
	isAuthenticationRoute := strings.Contains(info.FullMethod, "Authentification/PasswordLogin")
	if isAuthenticationRoute {
		return handler(ctx, req)
	}

	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return nil, fmt.Errorf("failed to get metadata from context")
	}

	authHeaderString, ok := md["authorization"]
	if !ok {
		return nil, fmt.Errorf("authorization header not found")
	}

	authHeader := strings.Split(authHeaderString[0], " ")

	if authHeader[0] != "Bearer" {
		return nil, fmt.Errorf("authorization header should start with Bearer")
	}

	claims, err := common.VerifyToken(authHeader[1])
	if err != nil {
		return nil, fmt.Errorf("failed to verify token: %w", err)
	}

	logger.Debug().Str("user", claims.Subject).Msg("Authentification interceptor")
	ctx = context.WithValue(ctx, "user", claims.Subject)

	return handler(ctx, req)
}
