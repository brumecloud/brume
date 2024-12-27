package fx_runner

import (
	docker "github.com/brumecloud/agent/container/docker"
	"github.com/brumecloud/agent/runner"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

var logger = log.With().Str("module", "runner").Logger()

var RunnerModule = fx.Module("runner",
	fx.Provide(NewRunner),
	fx.Invoke(func(runner runner.Runner) {}),
)

func NewRunner(lc fx.Lifecycle, dockerService *docker.DockerService) runner.Runner {
	logger.Info().Msg("Creating docker engine runner")

	// for the moment we only support docker runner
	return docker.NewDockerEngineRunner(dockerService)
}
