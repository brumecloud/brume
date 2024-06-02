package router

import (
	"context"
	"net"

	"github.com/brume/brume/internal/db"
	v1 "github.com/brume/brume/internal/gen/brume/v1"
	"github.com/brume/brume/internal/server"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

type Router struct{}

func NewRouter(lc fx.Lifecycle, db *db.DB) *Router {
	log.Info().Msg("Creating the router")

	grpcServer := grpc.NewServer()

	// Register services
	v1.RegisterAuthentificationServer(grpcServer, server.NewAuthentificationServer(db))

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

			log.Info().Msg("☁️  launched on port 9876")

			return nil
		},
		OnStop: func(context.Context) error {
			grpcServer.GracefulStop()
			log.Info().Msg("Brume ☁️  stopped")

			return nil
		},
	})

	return &Router{}
}
