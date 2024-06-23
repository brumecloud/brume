package graphql_router

import (
	"context"
	"io"
	"net"
	"net/http"
	"os"

	"brume.dev/account/user"
	resolver "brume.dev/internal/router/graphql/resolver"
	graphql "github.com/graph-gophers/graphql-go"
	"github.com/graph-gophers/graphql-go/relay"
	"github.com/rs/cors"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

// for fx
type GraphQLServer struct{}

func readGraphQLSchema() string {
	// open file scheam.graphql
	schemaFile, err := os.Open("./internal/router/graphql/schema.graphql")

	if err != nil {
		log.Panic().Err(err).Msg("Failed to open schema file")
		panic(err)
	}

	defer schemaFile.Close()
	schemaFileContent, err := io.ReadAll(schemaFile)

	if err != nil {
		log.Panic().Err(err).Msg("Failed to read schema file")
		panic(err)
	}

	return string(schemaFileContent)
}

func NewGraphQLServer(lc fx.Lifecycle, userService *user.UserService) *GraphQLServer {
	log.Info().Msg("Creating the GraphQL server")

	schemaFileContent := readGraphQLSchema()

	schema := graphql.MustParseSchema(schemaFileContent, &resolver.RootResolver{})

	cors := cors.New(cors.Options{
		AllowedOrigins:   []string{"*"},
		AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders:   []string{"Accept", "Authorization", "Content-Type", "X-CSRF-Token"},
		ExposedHeaders:   []string{"Link"},
		AllowCredentials: true,
		MaxAge:           300,
	})

	// graphql server
	http.Handle("/graphql", cors.Handler(&relay.Handler{
		Schema: schema,
	}))

	// graphiql server
	http.Handle("/", http.FileServer(http.Dir("./static/graphiql")))

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
