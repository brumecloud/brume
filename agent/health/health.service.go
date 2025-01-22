package health_service

import (
	"context"

	"github.com/brumecloud/agent/internal/config"
	intercom_service "github.com/brumecloud/agent/internal/intercom"
	"github.com/brumecloud/agent/job"
	"github.com/brumecloud/agent/runner"
	"github.com/brumecloud/agent/ticker"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

var logger = log.With().Str("module", "health").Logger()

type HealthService struct {
	runnerService *runner.RunnerService
	intercom      *intercom_service.IntercomService
	ticker        *ticker.Ticker
	cfg           *config.AgentConfig
}

// this service will send the health of the agent to the orchestrator
func NewHealthService(lc fx.Lifecycle, runnerService *runner.RunnerService, jobService *job.JobService, ticker *ticker.Ticker, intercom *intercom_service.IntercomService, cfg *config.AgentConfig) *HealthService {
	stopChannel := make(chan struct{})
	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			go func() {
				for {
					select {
					// we use the rapid ticker to update the orchestrator with the health of the agent
					// it also get the health of the running jobs
					case <-ticker.RapidTicker.C:
						var err error
						health, err := runnerService.GetRunnerHealth(context.Background())
						if err != nil {
							logger.Error().Err(err).Msg("Failed to get runner health")
						}

						err = intercom.SendHealth(health)
						if err != nil {
							logger.Error().Err(err).Msg("Failed to send health")
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

	return &HealthService{runnerService: runnerService, intercom: intercom, ticker: ticker, cfg: cfg}
}
