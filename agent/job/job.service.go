package job_service

import (
	"context"

	job_model "brume.dev/jobs/model"
	"github.com/brumecloud/agent/internal/config"
	intercom_service "github.com/brumecloud/agent/internal/intercom"
	"github.com/brumecloud/agent/ticker"
	"go.uber.org/fx"

	"github.com/rs/zerolog/log"
)

type JobService struct {
	cfg      *config.AgentConfig
	ticker   *ticker.Ticker
	intercom *intercom_service.IntercomService
}

var logger = log.With().Str("module", "job").Logger()

func NewJobService(lc fx.Lifecycle, cfg *config.AgentConfig, ticker *ticker.Ticker, intercom *intercom_service.IntercomService) *JobService {
	j := &JobService{
		cfg:      cfg,
		ticker:   ticker,
		intercom: intercom,
	}

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			logger.Info().Msg("Starting the job bidding loop")
			go j.Run(ctx)
			return nil
		},
	})

	return j
}

// this is the main loop of the agent
// it will poll the scheduler for a job
// once a job is received, it will start the runner
func (j *JobService) Run(ctx context.Context) error {
	ticker := j.ticker.SlowTicker

	for {
		select {
		case <-ticker.C:
			// each tick, we get all the available jobs
			go func() {
				// TODO: get the job from the scheduler
				jobs, err := j.intercom.GetJobs(ctx)
				if err != nil {
					logger.Error().Err(err).Msg("Failed to get job")
					return
				}

				for _, job := range jobs {
					go func(job *job_model.Job) {
						err := j.JobLifecycle(ctx, job)
						if err != nil {
							logger.Error().Err(err).Msg("Failed to process job")
						}
					}(job)
				}
			}()
		}
	}
}

func (j *JobService) JobLifecycle(ctx context.Context, job *job_model.Job) error {
	bid, err := j.ComputeBid(ctx, job)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to compute bid")
		return err
	}

	accepted, err := j.intercom.PlaceBid(ctx, job, bid)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to place bid")
		return err
	}

	if !accepted {
		logger.Warn().Msg("Bid not accepted")
		return nil
	}

	logger.Info().Msg("Bid accepted")

	return nil
}

func (j *JobService) ComputeBid(ctx context.Context, job *job_model.Job) (int, error) {
	return 1000, nil
}
