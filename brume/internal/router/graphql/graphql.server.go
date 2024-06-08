package graphql_router

import (
	"context"
	"net"
	"net/http"

	"brume.dev/internal/router/graphql/schema"
	"github.com/graphql-go/handler"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type GraphQLServer struct{}

func NewGraphQLServer(lc fx.Lifecycle) *GraphQLServer {
	log.Info().Msg("Creating the GraphQL server")
	schema, err := graphql_schema.NewSchema()

	if err != nil {
		panic(err)
	}

	h := handler.New(&handler.Config{
		Schema:     &schema,
		Pretty:     true,
		Playground: true,
		GraphiQL:   true,
	})

	http.Handle("/graphql", h)

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			log.Info().Msg("Launching GraphQL server")

			var lis net.Listener
			lis, err := net.Listen("tcp", "localhost:9877")

			if err != nil {
				panic(err)
			}

			go func() {
				if err := http.Serve(lis, nil); err != nil {
					panic(err)
				}
			}()

			log.Info().Msg("☁️  launched GraphQL on port 9877")

			return nil
		},
		OnStop: func(context.Context) error {
			log.Info().Msg("GraphQL server stopped")
			return nil
		},
	})

	return &GraphQLServer{}
}
