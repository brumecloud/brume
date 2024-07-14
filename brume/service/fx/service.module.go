package fx_service

import (
	"brume.dev/service"
	"go.uber.org/fx"
)

var ServiceModule = fx.Options(
	fx.Provide(service.NewServiceService),
	fx.Invoke(func(s *service.ServiceService) {}),
)
