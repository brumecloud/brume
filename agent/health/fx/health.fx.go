package health_fx

import (
	health_service "github.com/brumecloud/agent/health"
	"github.com/brumecloud/agent/internal/log"
	"go.uber.org/fx"
)

var logger = log.GetLogger("health")
var HealthModule = fx.Module("health",
	fx.Provide(health_service.NewHealthService),
	fx.Invoke(func(healthService *health_service.HealthService) {
		logger.Info().Msg("Health service started")
	}),
)
