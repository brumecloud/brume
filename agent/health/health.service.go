package health_service

import (
	"context"
	"os"

	"github.com/brumecloud/agent/internal/config"
	intercom_service "github.com/brumecloud/agent/internal/intercom"
	runner "github.com/brumecloud/agent/runner"
	"github.com/brumecloud/agent/ticker"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

var logger = log.With().Str("module", "health").Logger()

type HealthService struct {
	runner   runner.Runner
	intercom *intercom_service.IntercomService
	ticker   *ticker.Ticker
	cfg      *config.AgentConfig
}

// this service will send the health of the agent to the orchestrator
func NewHealthService(lc fx.Lifecycle, runner runner.Runner, ticker *ticker.Ticker, intercom *intercom_service.IntercomService, cfg *config.AgentConfig) *HealthService {
	stopChannel := make(chan struct{})
	logger.Info().Int("retryMax", cfg.RetryMax).Msg("Starting health service")
	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			go func() {
				errorCounter := 0
				for {
					select {
					// we use the rapid ticker to update the orchestrator with the health of the agent
					case <-ticker.RapidTicker.C:
						health, err := runner.GetRunnerHealth(context.Background())
						if err != nil {
							logger.Error().Err(err).Int("errorCounter", errorCounter).Msg("Failed to get runner health")
							errorCounter++

							if errorCounter > cfg.RetryMax && cfg.RetryMax != 0 {
								logger.Error().Msg("Health service is not healthy, stopping")
								os.Exit(1)
							}
						}

						err = intercom.SendGeneralHealth(health)

						if err != nil {
							logger.Error().Err(err).Int("errorCounter", errorCounter).Msg("Error while sending health")
							errorCounter++

							if errorCounter > cfg.RetryMax && cfg.RetryMax != 0 {
								logger.Error().Msg("Health service is not healthy, stopping")
								os.Exit(1)
							}
						} else {
							// full loop is healthy we reset the error counter
							errorCounter = 0
						}
					// if the stop channel is closed, we stop the health service
					case <-stopChannel:
						logger.Info().Msg("Received health service stop signal")
						return
					}
				}
			}()

			return nil
		},
		OnStop: func(context.Context) error {
			logger.Info().Msg("Health service stopped")
			close(stopChannel)
			return nil
		},
	})

	return &HealthService{runner: runner, intercom: intercom, ticker: ticker, cfg: cfg}
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
			logger.Error().Err(err).Msg("Failed to get runner health")
		}

		h.intercom.SendGeneralHealth(health)
	}
}
