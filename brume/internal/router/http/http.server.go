package http_router

import (
	"context"
	"fmt"
	"net/http"

	"brume.dev/account/org"
	"brume.dev/account/user"
	cloud_account_service "brume.dev/cloud/account"
	common "brume.dev/internal/common"
	config "brume.dev/internal/config"
	"brume.dev/internal/log"
	http_middleware "brume.dev/internal/router/http/middleware"
	public_graph "brume.dev/internal/router/public-gql/graph"
	public_graph_generated "brume.dev/internal/router/public-gql/graph/generated/generated.go"
	brume_workos "brume.dev/internal/workos"
	job_service "brume.dev/jobs/service"
	"brume.dev/machine"
	"brume.dev/project"
	"brume.dev/service"
	stack_service "brume.dev/stack"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/handler/extension"
	"github.com/99designs/gqlgen/graphql/handler/transport"
	"github.com/gorilla/mux"
	"github.com/gorilla/websocket"
	"go.uber.org/fx"
)

type BrumeHTTPServer struct{}

var logger = log.GetLogger("router.http")

func NewHTTPServer(
	lc fx.Lifecycle,
	authentificationService *common.AuthentificationService,
	userService *user.UserService,
	projectService *project.ProjectService,
	serviceService *service.ServiceService,
	machineService *machine.MachineService,
	bidService *job_service.BidService,
	workosClient *brume_workos.WorkOSClient,
	schedulerHTTPRouter *SchedulerHTTPRouterV1,
	monitoringHTTPRouter *MonitoringHTTPRouterV1,
	cloudAccountService *cloud_account_service.CloudAccountService,
	organizationService *org.OrganizationService,
	stackService *stack_service.StackService,
	cfg *config.BrumeConfig,
) *BrumeHTTPServer {
	logger.Info().Msg("Launching the HTTP Server")

	public_resolver := &public_graph.Resolver{
		ConfigService:       cfg,
		UserService:         userService,
		ProjectService:      projectService,
		ServiceService:      serviceService,
		MachineService:      machineService,
		CloudAccountService: cloudAccountService,
		OrganizationService: organizationService,
		StackService:        stackService,
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
	frontend_api_router := GeneralHTTPRouter(authentificationService, public_gql, workosClient, cfg)

	// api used to interact with the orchestrator
	orchestrator_server := mux.NewRouter()

	scheduler_router := orchestrator_server.PathPrefix("/scheduler/v1").Subrouter()
	schedulerHTTPRouter.RegisterRoutes(scheduler_router)

	monitoring_router := orchestrator_server.PathPrefix("/monitoring/v1").Subrouter()
	monitoringHTTPRouter.RegisterRoutes(monitoring_router)

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			go func() {
				listenAddr := fmt.Sprintf("%s:%d", cfg.ServerConfig.Host, cfg.ServerConfig.GraphqlPort)
				logger.Info().Str("listenAddr", listenAddr).Msg("Launching Public HTTP server")

				if err := http.ListenAndServe(listenAddr, http_middleware.CorsHandler.Handler(frontend_api_router)); err != nil {
					panic(err)
				}
			}()

			go func() {
				listenAddr := fmt.Sprintf("%s:%d", cfg.ServerConfig.Host, cfg.ServerConfig.OrchestratorPort)
				logger.Info().Str("listenAddr", listenAddr).Msg("Launching Orchestrator HTTP server")

				if err := http.ListenAndServe(listenAddr, http_middleware.CorsHandler.Handler(orchestrator_server)); err != nil {
					panic(err)
				}
			}()

			return nil
		},
		OnStop: func(context.Context) error {
			logger.Info().Msg("All servers stopped")
			return nil
		},
	})

	return &BrumeHTTPServer{}
}
