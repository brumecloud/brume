package http_router

import (
	"context"
	"net/http"

	"brume.dev/account/user"
	"brume.dev/internal/common"
	public_graph "brume.dev/internal/router/public-gql/graph"
	public_graph_generated "brume.dev/internal/router/public-gql/graph/generated/generated.go"
	brume_log "brume.dev/logs"
	"brume.dev/machine"
	"brume.dev/project"
	"brume.dev/service"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/handler/extension"
	"github.com/99designs/gqlgen/graphql/handler/transport"
	"github.com/gorilla/websocket"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type BrumeHTTPServer struct{}

func NewHTTPServer(lc fx.Lifecycle, authentificationService *common.AuthentificationService, userService *user.UserService, projectService *project.ProjectService, serviceService *service.ServiceService, logService *brume_log.LogService, machineService *machine.MachineService) *BrumeHTTPServer {
	log.Info().Msg("Launching the HTTP Server")

	public_resolver := &public_graph.Resolver{
		UserService:    userService,
		ProjectService: projectService,
		ServiceService: serviceService,
		LogService:     logService,
		MachineService: machineService,
	}

	public_gql := handler.New(public_graph_generated.NewExecutableSchema(public_graph_generated.Config{Resolvers: public_resolver}))

	public_gql.AddTransport(transport.SSE{})
	public_gql.AddTransport(transport.POST{})
	public_gql.AddTransport(transport.Websocket{Upgrader: websocket.Upgrader{
		CheckOrigin: func(r *http.Request) bool {
			return true
		},
	}})
	public_gql.Use(extension.Introspection{})

	general_router := GeneralHTTPRouter(authentificationService, public_gql)
	ingest_router := AgentHTTPRouterV1()

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {

			go func() {
				log.Info().Msg("Launching Public HTTP server on port 9877")
				if err := http.ListenAndServe("0.0.0.0:9877", general_router); err != nil {
					panic(err)
				}
			}()

			go func() {
				log.Info().Msg("Launching Ingest HTTP server on port 9876")
				if err := http.ListenAndServe("0.0.0.0:9876", ingest_router); err != nil {
					panic(err)
				}
			}()

			return nil
		},
		OnStop: func(context.Context) error {
			log.Info().Msg("All servers stopped")
			return nil
		},
	})

	return &BrumeHTTPServer{}
}
