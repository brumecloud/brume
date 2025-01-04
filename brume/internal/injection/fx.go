package injection

import (
	fx_org "brume.dev/account/org/fx"
	fx_user "brume.dev/account/user/fx"
	fx_builder "brume.dev/builder/fx"
	fx_deployment "brume.dev/deployment/fx"
	brume_clickhouse "brume.dev/internal/clickhouse"
	fx_common "brume.dev/internal/common/fx"
	config "brume.dev/internal/config"
	db "brume.dev/internal/db"
	fx_job "brume.dev/internal/jobs/fx"
	brumelog "brume.dev/internal/log"
	brume_redis "brume.dev/internal/redis"
	fx_grpc "brume.dev/internal/router/grpc/fx"
	fx_http "brume.dev/internal/router/http/fx"
	fx_temporal "brume.dev/internal/temporal/fx"
	temporal_worker "brume.dev/internal/temporal/worker"
	fx_log "brume.dev/logs/fx"
	fx_machine "brume.dev/machine/fx"
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

		fx_common.CommonModule,
		fx_org.OrgModule,
		fx_user.UserModule,
		fx_project.ProjectModule,
		fx_service.ServiceModule,
		fx_runner.RunnerModule,
		fx_builder.BuilderModule,
		fx_log.LogModule,
		fx_deployment.DeploymentModule,
		fx_temporal.TemporalOrchestratorModule,
		fx_machine.Module,
		fx_job.JobModule,
		config.ConfigModule,
		brume_clickhouse.ClickhouseModule,
		brume_redis.RedisModule,
		db.DBModule,

		fx_http.HttpModule,
		fx_grpc.GRPCModule,
		fx_temporal.TemporalModule,

		fx.Invoke(func(w *temporal_worker.MasterWorker) {}),
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Run() {
	log.Info().Msg("Running the application")

	g.Injector.Run()
}
