package job

import (
	"context"

	brume_job "brume.dev/jobs/model"
	runner_interfaces "github.com/brumecloud/agent/container/interfaces"
	"github.com/brumecloud/agent/internal/config"
	intercom_service "github.com/brumecloud/agent/internal/intercom"
	runner_service "github.com/brumecloud/agent/runner"
	"github.com/brumecloud/agent/ticker"
	"go.uber.org/fx"

	"github.com/rs/zerolog/log"
)

type JobService struct {
	cfg      *config.AgentConfig
	ticker   *ticker.Ticker
	intercom *intercom_service.IntercomService
	runner   *runner_service.RunnerService
}

var logger = log.With().Str("module", "job").Logger()

func NewJobService(lc fx.Lifecycle, runner *runner_service.RunnerService, cfg *config.AgentConfig, ticker *ticker.Ticker, intercom *intercom_service.IntercomService) *JobService {
	j := &JobService{
		cfg:      cfg,
		ticker:   ticker,
		intercom: intercom,
		runner:   runner,
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
	tick := 0

	for {
		select {
		case <-j.ticker.SlowTicker.C:
			j.SlowTickerRun(ctx, tick)
		case <-j.ticker.RapidTicker.C:
			j.FastTickerRun(ctx, tick)
		}
		tick++
	}
}

// do the health check and logs of all the running jobs
// this will send the status of the job and the status of the runner
func (j *JobService) FastTickerRun(ctx context.Context, tick int) error {
	// TODO find a way to avoid having the next tick firing before the current one is done

	runningJobs, err := j.runner.GetAllRunningJobs()
	if err != nil {
		logger.Error().Err(err).Msg("Failed to get all running jobs")
		return err
	}

	runningJobsStatus := make(map[string]brume_job.JobStatusEnum)
	for _, job := range runningJobs {
		runningJobsStatus[job.JobID] = job.Status
	}

	j.intercom.SendRunningJobsHealth(runningJobsStatus)
	return nil
}

// get the new jobs from the scheduler
// stop the old jobs
func (j *JobService) SlowTickerRun(ctx context.Context, tick int) {
	// each slow tick, we get all the new available jobs
	// we try to spawn them : bidding logic etc
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
			go func(job *brume_job.Job) {
				err := j.SpawnJob(ctx, job)
				if err != nil {
					logger.Error().Err(err).Msg("Failed to process job")
				}
			}(job)
		}
	}()

	go func() {
		runningJobs, err := j.runner.GetAllRunningJobs()
		if err != nil {
			logger.Error().Err(err).Msg("Failed to get running jobs")
			return
		}

		numberOfJobs := len(runningJobs)

		// get the job at index tick % numberOfJobs
		job := runner_interfaces.ContainerStatusResult{}
		index := 0
		// todo refactor this
		for _, job_ := range runningJobs {
			if index == tick%numberOfJobs {
				job = job_
				break
			}
			index++
		}

		if job.JobID == "" {
			return
		}

		orchestratorJobStatus, err := j.intercom.GetJobStatus(context.Background(), job.JobID)
		if err != nil {
			logger.Error().Err(err).Msg("Failed to get job status")
			return
		}

		// the orchestrator wants this job to be stopped
		// this is not urgent, this is a garbage collection
		if orchestratorJobStatus.Status == brume_job.JobStatusEnumStopped {
			j.ReleaseJob(job)
		}
	}()
}

// spawn new jobs
func (j *JobService) SpawnJob(ctx context.Context, job *brume_job.Job) error {
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

	_, err = j.runner.StartJob(ctx, job)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to start job")
		return err
	}

	logger.Info().Str("job_id", job.ID.String()).Msg("Job started")

	return nil
}

// TODO refactor
func (j *JobService) ReleaseJob(res runner_interfaces.ContainerStatusResult) error {
	// communicate first and then remove from memory
	err := j.intercom.ReleaseJob(context.Background(), res.JobID)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to release job")
		return err
	}

	// todo: do the proper cleanup
	// ie logs, images etc
	// err = j.runner.StopJob(context.Background(), job)
	// if err != nil {
	// 	logger.Error().Err(err).Msg("Failed to stop job")
	// 	return err
	// }

	return nil
}

func (j *JobService) ComputeBid(ctx context.Context, job *brume_job.Job) (int, error) {
	return 1000, nil
}
