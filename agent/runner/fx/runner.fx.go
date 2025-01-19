package fx_runner

import (
	runner "github.com/brumecloud/agent/runner"
	"go.uber.org/fx"
)

var RunnerModule = fx.Module("runner",
	fx.Provide(runner.NewRunnerService),
)
