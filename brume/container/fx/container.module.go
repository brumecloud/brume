package fx_container

import (
	"brume.dev/container"
	"go.uber.org/fx"
)

var ContainerModule = fx.Module("container",
	fx.Provide(container.NewContainerActivity),
	fx.Invoke(func(activity *container.ContainerActivity) {}),
)
