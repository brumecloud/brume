package injection

import (
	fx_health "agent.brume.dev/health/fx"
	fx_config "agent.brume.dev/internal/config"
	fx_intercom "agent.brume.dev/internal/intercom/fx"
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
		fx_config.ConfigModule,
		fx_intercom.IntercomModule,
		fx_health.HealthModule,
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
