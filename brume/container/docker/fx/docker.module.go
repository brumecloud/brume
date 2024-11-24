package fx_docker

import (
	"brume.dev/container/docker"

	"go.uber.org/fx"
)

var Module = fx.Module("docker",
	fx.Provide(docker.NewDockerService),
	fx.Provide(docker.NewDockerActivity),
	fx.Invoke(func(s *docker.DockerService, a *docker.DockerActivity) {}),
)
