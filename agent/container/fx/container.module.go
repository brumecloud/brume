package fx_container

import (
	"github.com/brumecloud/agent/container/docker"
	"go.uber.org/fx"
)

var ContainerModule = fx.Module("container",
	fx.Provide(docker.NewDockerEngineRunner),
)
