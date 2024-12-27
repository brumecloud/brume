package fx_runner

import (
	docker "github.com/brumecloud/agent/container/docker"
	"github.com/brumecloud/agent/runner"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

var RunnerModule = fx.Module("runner",
	fx.Provide(NewRunner),
	fx.Invoke(func(runner *docker.DockerEngineRunner) {}),
)

func NewRunner(lc fx.Lifecycle, dockerService *docker.DockerService) runner.Runner {
	logger := log.With().Str("module", "runner").Logger()

	logger.Info().Msg("Creating docker engine runner")

	// for the moment we only support docker runner
	return docker.NewDockerEngineRunner(dockerService)
}
