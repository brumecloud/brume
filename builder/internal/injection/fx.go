package injection

import (
	"github.com/ipfans/fxlogger"

	fx_config "github.com/brumecloud/builder/internal/config"
	"github.com/brumecloud/builder/internal/log"
	"go.uber.org/fx"
)

var logger = log.GetLogger("injection")

type GlobalInjector struct {
	Injector *fx.App
}

func NewBuilderInjector() *GlobalInjector {
	logger.Info().Msg("Initializing builder injector")

	app := fx.New(
		fx.WithLogger(fxlogger.WithZerolog(logger)),
		fx_config.ConfigModule,
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Run() {
	logger.Info().Msg("Running builder")
	g.Injector.Run()
}
