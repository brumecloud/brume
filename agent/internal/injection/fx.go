package injection

import (
	fx_health "github.com/brumecloud/agent/health/fx"
	fx_config "github.com/brumecloud/agent/internal/config"
	fx_intercom "github.com/brumecloud/agent/internal/intercom/fx"
	brumelog "github.com/brumecloud/agent/internal/log"
	fx_ticker "github.com/brumecloud/agent/ticker"
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
