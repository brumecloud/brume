package health_fx

import (
	health_service "agent.brume.dev/health"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

var HealthModule = fx.Module("health",
	fx.Provide(health_service.NewHealthService),
	fx.Invoke(func(healthService *health_service.HealthService) {
		log.Info().Msg("Health service started")
	}),
)
