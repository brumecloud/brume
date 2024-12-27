package health_service

import (
	"context"

	runner_interfaces "agent.brume.dev/container/runner/interfaces"
	intercom_service "agent.brume.dev/internal/intercom"
	"agent.brume.dev/ticker"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type HealthService struct {
	runner   runner_interfaces.ContainerRunner
	intercom *intercom_service.IntercomService
	ticker   *ticker.Ticker
}

func NewHealthService(lc fx.Lifecycle, runner runner_interfaces.ContainerRunner, ticker *ticker.Ticker, intercom *intercom_service.IntercomService) *HealthService {
	stopChannel := make(chan struct{})
	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			go func() {
				select {
				// we use the rapid ticker to update the orchestrator with the health of the agent
				case <-ticker.RapidTicker.C:
					health, err := runner.GetRunnerHealth(context.Background())

					if err != nil {
						log.Error().Err(err).Msg("Failed to get runner health")
					}

					intercom.SendGeneralHealth(health)
				// if the stop channel is closed, we stop the health service
				case <-stopChannel:
					log.Info().Msg("Received health service stop signal")
					return
				}
			}()

			return nil
		},
		OnStop: func(context.Context) error {
			close(stopChannel)
			log.Info().Msg("Health service stopped")
			return nil
		},
	})

	return &HealthService{runner: runner, intercom: intercom, ticker: ticker}
}

// This is the main health function for the agent
// if the agent is healthy, it will return true
// if the agent is not healthy, it will return false
// when an agent is not healthy, a new job will be created
// on an healthy agent
func (h *HealthService) AgentHealth() {
	select {
	// we use the rapid ticker to update the orchestrator with the health of the agent
	case <-h.ticker.RapidTicker.C:
		health, err := h.runner.GetRunnerHealth(context.Background())

		if err != nil {
			log.Error().Err(err).Msg("Failed to get runner health")
		}

		h.intercom.SendGeneralHealth(health)
	}
}
