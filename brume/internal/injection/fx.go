package injection

import (
	fx_org "brume.dev/account/org/fx"
	fx_user "brume.dev/account/user/fx"
	fx_builder "brume.dev/builder/fx"
	fx_docker "brume.dev/container/docker/fx"
	fx_common "brume.dev/internal/common/fx"
	"brume.dev/internal/db"
	brumelog "brume.dev/internal/log"
	fx_grpc "brume.dev/internal/router/grpc/fx"
	fx_http "brume.dev/internal/router/http/fx"
	fx_temporal "brume.dev/internal/temporal/fx"
	"brume.dev/internal/temporal/worker"
	fx_log "brume.dev/logs/fx"
	fx_project "brume.dev/project/fx"
	fx_runner "brume.dev/runner/fx"
	fx_service "brume.dev/service/fx"

	"github.com/ipfans/fxlogger"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type GlobalInjector struct {
	Injector *fx.App
}

func NewMasterInjector() *GlobalInjector {
	log.Info().Msg("Initializing master injector")

	app := fx.New(
		fx.WithLogger(fxlogger.WithZerolog(brumelog.GetLogger())),
		fx.Provide(db.InitDB),
		fx.Invoke(db.InitDB),

		fx_common.CommonModule,
		fx_org.OrgModule,
		fx_user.UserModule,
		fx_project.ProjectModule,
		fx_service.ServiceModule,
		fx_runner.RunnerModule,
		fx_builder.BuilderModule,
		fx_log.LogModule,

		fx_http.HttpModule,
		fx_grpc.GRPCModule,
		fx_temporal.TemporalModule,
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func NewNodeInjector() *GlobalInjector {
	log.Info().Msg("Initializing node injector")

	app := fx.New(
		fx.WithLogger(fxlogger.WithZerolog(brumelog.GetLogger())),
		fx_temporal.TemporalModule,
		fx_temporal.TemporalNodeModule,
		fx_docker.Module,

		// Start the node worker
		fx.Invoke(func(w *temporal_worker.NodeWorker) {}),
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Run() {
	log.Info().Msg("Running the application")

	g.Injector.Run()
}
