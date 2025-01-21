package job

import (
	"context"
	"sync"
	"time"

	brume_job "brume.dev/jobs/model"
	"github.com/brumecloud/agent/internal/config"
	"github.com/brumecloud/agent/internal/db"
	intercom_service "github.com/brumecloud/agent/internal/intercom"
	running_job "github.com/brumecloud/agent/job/model"
	runner_service "github.com/brumecloud/agent/runner"
	"github.com/brumecloud/agent/ticker"
	"go.uber.org/fx"

	"github.com/rs/zerolog/log"
)

type JobService struct {
	cfg      *config.AgentConfig
	ticker   *ticker.Ticker
	intercom *intercom_service.IntercomService
	db       *db.DB
	runner   *runner_service.RunnerService
}

var logger = log.With().Str("module", "job").Logger()

func NewJobService(lc fx.Lifecycle, db *db.DB, runner *runner_service.RunnerService, cfg *config.AgentConfig, ticker *ticker.Ticker, intercom *intercom_service.IntercomService) *JobService {
	j := &JobService{
		cfg:      cfg,
		ticker:   ticker,
		intercom: intercom,
		db:       db,
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

	return nil
}

func (j *JobService) GetRunningJobs() ([]*running_job.RunningJob, error) {
	var jobs []*running_job.RunningJob
	err := j.db.Gorm.Find(&jobs).Error
	logger.Info().Interface("jobs", jobs).Msg("Found running jobs")
	return jobs, err
}

func (j *JobService) AddRunningJob(job *running_job.RunningJob) {
	logger.Info().Interface("job", job).Msg("Adding running job")
	j.db.Gorm.Create(job)
}

func (j *JobService) RemoveRunningJob(job *running_job.RunningJob) {
	logger.Info().Interface("job", job).Msg("Removing running job")
	j.db.Gorm.Delete(job)
}

// do the health check and logs of all the running jobs
// this will send the status of the job and the status of the runner
func (j *JobService) FastTickerRun(ctx context.Context, tick int) error {
	// TODO find a way to avoid having the next tick firing before the current one is done
	jobs, err := j.GetRunningJobs()
	if err != nil {
		logger.Error().Err(err).Msg("Failed to get running jobs")
		return err
	}

	jobs_status := make(map[string]brume_job.JobStatusEnum)

	wg := sync.WaitGroup{}
	for _, job := range jobs {
		// go routine to get the status of the job
		wg.Add(1)
		go func(job *running_job.RunningJob) {
			defer wg.Done()
			status, err := j.runner.GetJobStatus(ctx, job)
			if err != nil {
				logger.Error().Err(err).Msg("Failed to get job status")
				jobs_status[job.ID] = "failed"
			}

			jobs_status[job.ID] = status

			logger.Info().Str("job_id", job.ID).Str("status", string(status)).Msg("Job status")
		}(job)
	}

	wg.Wait()

	j.intercom.SendRunningJobsHealth(jobs_status)
	return nil
}

// get the new jobs from the scheduler
// stop the old jobs
func (j *JobService) SlowTickerRun(ctx context.Context, tick int) {
	jobs, err := j.GetRunningJobs()
	if err != nil {
		logger.Error().Err(err).Msg("Failed to get running jobs")
		return
	}

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
			go func(job *brume_job.Job) {
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
		if len(jobs) == 0 {
			return
		}

		job := jobs[tick%len(jobs)]
		jobStatus, err := j.intercom.GetJobStatus(ctx, job.ID)
		if err != nil {
			logger.Error().Err(err).Msg("Failed to get job status")
		}

		if jobStatus.Status == brume_job.JobStatusEnumStopped {
			// TODO: remove the job from the list of the running jobs
			j.ReleaseJob(job)
			logger.Info().Str("job_id", job.ID).Msg("Job stopped by the orchestrator")
		}
	}()
}

func (j *JobService) JobLifecycle(ctx context.Context, job *brume_job.Job) error {
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

	containerId, err := j.runner.StartJob(ctx, job.Deployment)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to start job")
		return err
	}

	// appending the job to the running job list will put it on the status checking list
	// that way the orchestrator will be informed of the status of the job
	runningJob := &running_job.RunningJob{
		ID:           job.ID.String(),
		DeploymentID: job.Deployment.ID.String(),
		JobType:      running_job.DockerRunningJob,
		ContainerID:  &containerId,
		LastCheckAt:  time.Now(),
	}

	j.AddRunningJob(runningJob)

	logger.Info().Str("job_id", job.ID.String()).Msg("Job started")

	return nil
}

func (j *JobService) ReleaseJob(job *running_job.RunningJob) error {
	// communicate first and then remove from memory
	err := j.intercom.ReleaseJob(context.Background(), job.ID)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to release job")
		return err
	}

	// todo: do the proper cleanup
	// ie logs, images etc
	err = j.runner.StopJob(context.Background(), job)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to stop job")
		return err
	}

	// remove the job from memory
	j.RemoveRunningJob(job)

	return nil
}

func (j *JobService) ComputeBid(ctx context.Context, job *brume_job.Job) (int, error) {
	return 1000, nil
}
