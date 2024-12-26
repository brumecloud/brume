package health_fx

import (
	health_service "agent.brume.dev/health"
	"go.uber.org/fx"
)

var HealthModule = fx.Module("health",
	fx.Provide(health_service.NewHealthService),
)
