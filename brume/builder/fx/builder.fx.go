package builder_fx

import (
	builder_service "brume.dev/builder/service"
	"go.uber.org/fx"
)

var BuilderModule = fx.Options(
	fx.Provide(builder_service.NewBuilderService),
	fx.Invoke(func(s *builder_service.BuilderService) {}),
)
