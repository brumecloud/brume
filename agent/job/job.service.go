package job_service

import (
	"context"

	job_model "brume.dev/jobs/model"
	"github.com/brumecloud/agent/internal/config"
	intercom_service "github.com/brumecloud/agent/internal/intercom"
	runner_service "github.com/brumecloud/agent/runner"
	"github.com/brumecloud/agent/ticker"
	"go.uber.org/fx"

	"github.com/rs/zerolog/log"
)

type JobService struct {
	cfg         *config.AgentConfig
	ticker      *ticker.Ticker
	intercom    *intercom_service.IntercomService
	runningJobs []*job_model.Job
	runner      *runner_service.RunnerService
}

var logger = log.With().Str("module", "job").Logger()

func NewJobService(lc fx.Lifecycle, runner *runner_service.RunnerService, cfg *config.AgentConfig, ticker *ticker.Ticker, intercom *intercom_service.IntercomService) *JobService {
	j := &JobService{
		cfg:         cfg,
		ticker:      ticker,
		intercom:    intercom,
		runningJobs: []*job_model.Job{},
		runner:      runner,
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
	tick := 0

	for {
		tick++
		select {
		case <-ticker.C:
			j.SlowTickerRun(ctx, tick)
		case <-j.ticker.RapidTicker.C:
			j.FastTickerRun(ctx, tick)
		}
	}
}

// do the health check and logs of all the running jobs
// this will send the status of the job and the status of the runner
func (j *JobService) FastTickerRun(ctx context.Context, tick int) {
	go func() {
		for _, job := range j.runningJobs {
			_, err := j.runner.GetJobStatus(ctx, job.Deployment)
			if err != nil {
				logger.Error().Err(err).Msg("Failed to get job status")
			}
		}
	}()
}

// get the new jobs from the scheduler
// stop the old jobs
func (j *JobService) SlowTickerRun(ctx context.Context, tick int) {
	// each tick, we get all the available jobs
	go func() {
		// TODO: get the job from the scheduler
		jobs, err := j.intercom.GetJobs(ctx)
		if err != nil {
			logger.Error().Err(err).Msg("Failed to get job")
			return
		}

		if len(jobs) > 0 {
			logger.Info().Int("jobs", len(jobs)).Msg("Received jobs")
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

	// we do one tick per job to get if the job is stopped
	// by the orchestrator
	go func() {
		if len(j.runningJobs) == 0 {
			return
		}

		job := j.runningJobs[tick%len(j.runningJobs)]
		jobStatus, err := j.intercom.GetJobStatus(ctx, job.ID.String())
		if err != nil {
			logger.Error().Err(err).Msg("Failed to get job status")
		}

		if jobStatus.Status == job_model.JobStatusEnumStopped {
			// TODO: remove the job from the list of the running jobs
			j.ReleaseJob(job)
			logger.Info().Str("job_id", job.ID.String()).Msg("Job stopped by the orchestrator")
		}
	}()
}

func (j *JobService) JobLifecycle(ctx context.Context, job *job_model.Job) error {
	logger.Info().Str("job_id", job.ID.String()).Msg("Starting job lifecycle")

	bid, err := j.ComputeBid(ctx, job)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to compute bid")
		return err
	}

	// depending on the job, and how the runner is confident in running it
	// well. we place a bid on the job
	logger.Info().Int("bid", bid).Str("job_id", job.ID.String()).Msg("Placing bid")

	accepted, err := j.intercom.PlaceBid(ctx, job, bid)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to place bid")
		return err
	}

	if !accepted {
		logger.Warn().Msg("Bid not accepted by the orchestrator")
		return nil
	}

	logger.Info().Str("job_id", job.ID.String()).Msg("Bid accepted")

	// add the job to the list of the running jobs on the agent
	logger.Info().Str("job_id", job.ID.String()).Interface("deployment", job.Deployment).Msg("Try starting the job")

	err = j.runner.StartJob(ctx, job.Deployment)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to start job")
		return err
	}

	// appending the job to the running job list will put it on the status checking list
	// that way the orchestrator will be informed of the status of the job
	j.runningJobs = append(j.runningJobs, job)

	logger.Info().Str("job_id", job.ID.String()).Msg("Job started")

	return nil
}

func (j *JobService) ReleaseJob(job *job_model.Job) error {
	// communicate first and then remove from memory
	err := j.intercom.ReleaseJob(context.Background(), job.ID.String())
	if err != nil {
		logger.Error().Err(err).Msg("Failed to release job")
		return err
	}
	// todo: do the proper cleanup
	// ie logs, images etc
	err = j.runner.StopJob(context.Background(), job.Deployment)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to stop job")
		return err
	}

	// remove the job from memory
	j.runningJobs = removeJobFromList(j.runningJobs, job)

	return nil
}

func removeJobFromList(list []*job_model.Job, job *job_model.Job) []*job_model.Job {
	for i, j := range list {
		if j.ID == job.ID {
			return append(list[:i], list[i+1:]...)
		}
	}
	return list
}

func (j *JobService) ComputeBid(ctx context.Context, job *job_model.Job) (int, error) {
	return 1000, nil
}
