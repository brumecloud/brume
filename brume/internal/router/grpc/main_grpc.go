package grpc_router

import (
	"context"
	"fmt"
	"net"

	"brume.dev/account/org"
	"brume.dev/internal/common"
	"brume.dev/internal/config"
	v1 "brume.dev/internal/gen/brume/v1"
	interceptor "brume.dev/internal/router/grpc/interceptor"
	server "brume.dev/internal/router/grpc/server"

	brume_log "brume.dev/internal/log"
	"go.uber.org/fx"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

var logger = brume_log.GetLogger("router.grpc")

type GRPCRouter struct{}

func NewGRPCRouter(lc fx.Lifecycle, authService *common.AuthentificationService, orgService *org.OrganizationService, cfg *config.BrumeConfig) *GRPCRouter {
	logger.Info().Msg("Creating the gRPC router")

	grpcServer := grpc.NewServer(grpc.UnaryInterceptor(interceptor.AuthentificationInterceptor))

	// Register services
	v1.RegisterAuthentificationServer(grpcServer, server.NewGRPCAuthentificationServer(authService))
	v1.RegisterOrganizationServiceServer(grpcServer, server.NewOrganizationServer(orgService))

	reflection.Register(grpcServer)

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			var lis net.Listener
			listenAddr := fmt.Sprintf("%s:%d", cfg.ServerConfig.Host, cfg.ServerConfig.GRPCPort)

			logger.Info().Str("listenAddr", listenAddr).Msg("Launching gRPC server")

			lis, err := net.Listen("tcp", listenAddr)
			if err != nil {
				panic(err)
			}

			go func() {
				if err := grpcServer.Serve(lis); err != nil {
					panic(err)
				}
			}()

			logger.Info().Str("listenAddr", listenAddr).Msg("gRPC server launched")

			return nil
		},
		OnStop: func(context.Context) error {
			grpcServer.GracefulStop()
			logger.Info().Msg("gRPC server stopped")

			return nil
		},
	})

	return &GRPCRouter{}
}
