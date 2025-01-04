package http_router

import (
	"context"
	"fmt"
	"net/http"

	"brume.dev/account/user"
	"brume.dev/internal/common"
	config "brume.dev/internal/config"
	job_service "brume.dev/internal/jobs/service"
	http_middleware "brume.dev/internal/router/http/middleware"
	public_graph "brume.dev/internal/router/public-gql/graph"
	public_graph_generated "brume.dev/internal/router/public-gql/graph/generated/generated.go"
	brume_log "brume.dev/logs"
	"brume.dev/machine"
	"brume.dev/project"
	"brume.dev/service"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/handler/extension"
	"github.com/99designs/gqlgen/graphql/handler/transport"
	"github.com/gorilla/mux"
	"github.com/gorilla/websocket"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type BrumeHTTPServer struct{}

func NewHTTPServer(
	lc fx.Lifecycle,
	authentificationService *common.AuthentificationService,
	userService *user.UserService,
	projectService *project.ProjectService,
	serviceService *service.ServiceService,
	logService *brume_log.LogService,
	machineService *machine.MachineService,
	bidService *job_service.BidService,
	schedulerHTTPRouter *SchedulerHTTPRouterV1,
	monitoringHTTPRouter *MonitoringHTTPRouterV1,
	cfg *config.BrumeConfig,
) *BrumeHTTPServer {
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
			// TODO: check if the origin is allowed
			return true
		},
	}})
	public_gql.Use(extension.Introspection{})

	// api used to interact with brume interface
	frontend_api_router := GeneralHTTPRouter(authentificationService, public_gql)

	// api used to interact with the orchestrator
	orchestrator_server := mux.NewRouter()

	scheduler_router := orchestrator_server.PathPrefix("/scheduler/v1").Subrouter()
	schedulerHTTPRouter.RegisterRoutes(scheduler_router)

	monitoring_router := orchestrator_server.PathPrefix("/monitoring/v1").Subrouter()
	monitoringHTTPRouter.RegisterRoutes(monitoring_router)

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			go func() {
				listenAddr := fmt.Sprintf("%s:%d", cfg.HTTPHost, cfg.HTTPPort)
				log.Info().Str("listenAddr", listenAddr).Msg("Launching Public HTTP server")

				if err := http.ListenAndServe(listenAddr, http_middleware.CorsHandler.Handler(frontend_api_router)); err != nil {
					panic(err)
				}
			}()

			go func() {
				listenAddr := fmt.Sprintf("%s:%d", cfg.HTTPHost, cfg.HTTPPort)
				log.Info().Str("listenAddr", listenAddr).Msg("Launching Orchestrator HTTP server")

				if err := http.ListenAndServe(listenAddr, http_middleware.CorsHandler.Handler(orchestrator_server)); err != nil {
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
