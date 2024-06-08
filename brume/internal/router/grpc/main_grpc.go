package grpc_router

import (
	"context"
	"net"

	"brume.dev/account/org"
	"brume.dev/internal/common"
	v1 "brume.dev/internal/gen/brume/v1"
	interceptor "brume.dev/internal/router/grpc/interceptor"
	server "brume.dev/internal/router/grpc/server"

	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

type GRPCRouter struct{}

func NewGRPCRouter(lc fx.Lifecycle, authService *common.AuthentificationService, orgService *org.OrganizationService) *GRPCRouter {
	log.Info().Msg("Creating the gRPC router")

	grpcServer := grpc.NewServer(grpc.UnaryInterceptor(interceptor.AuthentificationInterceptor))

	// Register services
	v1.RegisterAuthentificationServer(grpcServer, server.NewGRPCAuthentificationServer(authService))
	v1.RegisterOrganizationServiceServer(grpcServer, server.NewOrganizationServer(orgService))

	reflection.Register(grpcServer)

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			log.Info().Msg("Launching gRPC server")

			var lis net.Listener
			lis, err := net.Listen("tcp", "localhost:9876")

			if err != nil {
				panic(err)
			}

			go func() {
				if err := grpcServer.Serve(lis); err != nil {
					panic(err)
				}
			}()

			log.Info().Msg("☁️  launched gRPC on port 9876")

			return nil
		},
		OnStop: func(context.Context) error {
			grpcServer.GracefulStop()
			log.Info().Msg("gRPC server stopped")

			return nil
		},
	})

	return &GRPCRouter{}
}
