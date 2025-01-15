package job_service

import (
	"context"

	"github.com/brumecloud/agent/internal/config"
	"github.com/brumecloud/agent/ticker"
)

type JobService struct {
	cfg    *config.AgentConfig
	ticker *ticker.Ticker
}

func NewJobService(cfg *config.AgentConfig, ticker *ticker.Ticker) *JobService {
	return &JobService{
		cfg:    cfg,
		ticker: ticker,
	}
}

// this is the main loop of the agent
// it will poll the scheduler for a job
// once a job is received, it will start the runner
func (j *JobService) Run(ctx context.Context) error {
	ticker := j.ticker.SlowTicker

	for {
		select {
		case <-ticker.C:
			// TODO: get the job from the scheduler
		}
	}
}
