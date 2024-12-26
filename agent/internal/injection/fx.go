package injection

import (
	fx_runner "agent.brume.dev/container/runner/fx"
	fx_health "agent.brume.dev/health/fx"
	fx_intercom "agent.brume.dev/internal/intercom/fx"
	fx_job "agent.brume.dev/internal/jobs/fx"
	brumelog "agent.brume.dev/internal/log"
	fx_ticker "agent.brume.dev/ticker"
	"github.com/ipfans/fxlogger"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type GlobalInjector struct {
	Injector *fx.App
}

func NewAgentInjector() *GlobalInjector {
	log.Info().Msg("Initializing agent injector")

	app := fx.New(
		fx.WithLogger(fxlogger.WithZerolog(brumelog.GetLogger())),
		fx_intercom.IntercomModule,
		fx_health.HealthModule,
		fx_job.JobModule,
		fx_runner.RunnerModule,
		fx_ticker.TickerModule,
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Run() {
	log.Info().Msg("Running the agent")

	g.Injector.Run()
}
