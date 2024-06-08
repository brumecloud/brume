package injection

import (
	"brume.dev/internal/db"
	brumelog "brume.dev/internal/log"
	"brume.dev/internal/router"
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
		fx.Provide(router.NewRouter),
		fx.Invoke(func(router *router.Router) {}),
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Run() {
	log.Info().Msg("Running the application")

	g.Injector.Run()
}
