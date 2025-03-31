package injection

import (
	fx_collector "github.com/brumecloud/agent/collector/fx"
	fx_docker "github.com/brumecloud/agent/container/docker"
	fx_container "github.com/brumecloud/agent/container/fx"
	fx_health "github.com/brumecloud/agent/health/fx"
	fx_config "github.com/brumecloud/agent/internal/config"
	fx_intercom "github.com/brumecloud/agent/internal/intercom/fx"
	brumelog "github.com/brumecloud/agent/internal/log"
	fx_job "github.com/brumecloud/agent/job/fx"
	fx_runner "github.com/brumecloud/agent/runner/fx"
	fx_ticker "github.com/brumecloud/agent/ticker"
	"github.com/ipfans/fxlogger"
	"go.uber.org/fx"
)

var logger = brumelog.GetLogger("fx")

type GlobalInjector struct {
	Injector *fx.App
}

func NewAgentInjector() *GlobalInjector {
	logger.Info().Msg("Initializing agent injector")

	app := fx.New(
		fx.WithLogger(fxlogger.WithZerolog(logger)),
		fx_config.ConfigModule,
		fx_docker.DockerModule,
		fx_intercom.IntercomModule,
		fx_health.HealthModule,
		fx_runner.RunnerModule,
		fx_ticker.TickerModule,
		fx_container.ContainerModule,
		fx_job.JobModule,
		fx_collector.CollectorModule,
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Run() {
	logger.Info().Msg("Running the agent")
	g.Injector.Run()
}
