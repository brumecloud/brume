package injection

import (
	"brume.dev/internal/db"
	brumelog "brume.dev/internal/log"
	"github.com/ipfans/fxlogger"
	"go.uber.org/fx"
)

type GlobalInjector struct {
	Injector *fx.App
}

func NewGlobalInjector() *GlobalInjector {
	app := fx.New(
		fx.Invoke(brumelog.InitLogger),
		fx.WithLogger(fxlogger.WithZerolog(brumelog.GetLogger())),
		fx.Invoke(db.InitDB),
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Run() {
	g.Injector.Run()
}
