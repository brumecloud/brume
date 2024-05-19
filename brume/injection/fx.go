package injection

import (
	"go.uber.org/fx"
	"go.uber.org/fx/fxevent"
	"go.uber.org/zap"
)

type GlobalInjector struct {
	Injector *fx.App
}

func NewGlobalInjector() *GlobalInjector {
	app := fx.New(
		fx.WithLogger(func(log *zap.Logger) fxevent.Logger {
			return &fxevent.ZapLogger{Logger: log}
		}),
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Launch() {
	g.Injector.Run()
}
