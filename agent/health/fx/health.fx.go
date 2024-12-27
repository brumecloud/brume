package health_fx

import (
	health_service "github.com/brumecloud/agent/health"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

var HealthModule = fx.Module("health",
	fx.Provide(health_service.NewHealthService),
	fx.Invoke(func(healthService *health_service.HealthService) {
		log.Info().Msg("Health service started")
	}),
)
