package fx_executor

import (
	"brume.dev/executor"
	"go.uber.org/fx"
)

var ExecutorService = fx.Options(
	fx.Provide(executor.NewExecutorService),
	fx.Invoke(func(e *executor.ExecutorService) {}),
)
