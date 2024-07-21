package fx_executor

import (
	"brume.dev/runner"
	"go.uber.org/fx"
)

var RunnerModule = fx.Options(
	fx.Provide(runner.NewRunnerService),
	fx.Invoke(func(e *runner.RunnerService) {}),
)
