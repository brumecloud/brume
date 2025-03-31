package job_service

import (
	"context"
	"fmt"
	"sync"
	"time"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	temporal_constants "brume.dev/internal/temporal/constants"
	ticker "brume.dev/internal/ticker"
	job_model "brume.dev/jobs/model"
	"github.com/google/uuid"
	"github.com/redis/go-redis/v9"
	"go.temporal.io/sdk/client"
	"go.uber.org/fx"
)

const (
	JobHealthKey = "job:health:%s"
	JobStatusKey = "job:status:%s"
)

var jobLogger = log.GetLogger("job_service")

type JobService struct {
	redisClient    *redis.Client
	ticker         *ticker.TickerService
	db             *db.DB
	temporalClient client.Client
}

func NewJobService(lc fx.Lifecycle, redisClient *redis.Client, ticker *ticker.TickerService, db *db.DB, temporalClient client.Client) *JobService {
	js := &JobService{redisClient: redisClient, ticker: ticker, db: db, temporalClient: temporalClient}

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			jobLogger.Info().Msg("Starting the job health loop")
			go js.RunHealthLoop()
			return nil
		},
		OnStop: func(ctx context.Context) error {
			// TODO: use context to stop the main loop
			return nil
		},
	})

	return js
}

func (s *JobService) CreateJob(deployment *deployment_model.Deployment, workflowID string, runID string) (*job_model.Job, error) {
	job := &job_model.Job{
		ID:                   uuid.New(),
		Price:                9999999999,
		Status:               job_model.JobStatusEnumCreating,
		ServiceID:            deployment.ServiceID,
		Deployment:           deployment,
		DeploymentWorkflowID: workflowID,
		DeploymentRunID:      runID,
	}

	// set the job status to creating
	s.redisClient.Set(context.Background(), fmt.Sprintf(JobStatusKey, job.ID.String()), string(job_model.JobStatusEnumCreating), 0).Err()

	return job, s.db.Gorm.Create(job).Error
}

// store the status of the job in redis
// the health of a job has a ttl of 10 seconds
// thus you can only set the health of a job if it is healthy
func (s *JobService) SetJobHealth(jobID string) error {
	return s.redisClient.Set(context.Background(), fmt.Sprintf(JobHealthKey, jobID), "OK", 10*time.Second).Err()
}

// get the status of the job from redis
func (s *JobService) GetJobHealth(jobID string) (job_model.JobStatusEnum, error) {
	// we do not care about the value, we just want to check if the key exists
	_, err := s.redisClient.Get(context.Background(), fmt.Sprintf(JobHealthKey, jobID)).Result()
	if err != nil {
		return job_model.JobStatusEnumFailed, err
	}
	return job_model.JobStatusEnumRunning, nil
}

// set the job status in the database
func (s *JobService) SetJobStatus(jobID uuid.UUID, status job_model.JobStatusEnum) error {
	err := s.db.Gorm.Model(&job_model.Job{
		ID: jobID,
	}).Update("status", status).Error
	jobLogger.Trace().Str("job_id", jobID.String()).Str("status", string(status)).Msg("Setting job status")
	return err
}

func (s *JobService) SetJobContainerID(jobID uuid.UUID, containerID string) error {
	err := s.db.Gorm.Model(&job_model.Job{
		ID:          jobID,
		ContainerID: &containerID,
	}).Update("container_id", containerID).Error
	jobLogger.Trace().Str("job_id", jobID.String()).Str("container_id", containerID).Msg("Setting job container id")
	return err
}

// get the status of the job from the database
func (s *JobService) GetJobStatus(jobID uuid.UUID) (job_model.JobStatusEnum, error) {
	var job job_model.Job
	err := s.db.Gorm.Where("id = ?", jobID).First(&job).Error
	return job.Status, err
}

// do the actual job health check
func (s *JobService) WatchJob(job job_model.Job) bool {
	lastStatus, err := s.GetJobHealth(job.ID.String())
	if err != nil {
		jobLogger.Error().Err(err).Str("job_id", job.ID.String()).Msg("Failed to get the job health")
		return false
	}

	// if we got one good health check, we set the job status to running
	if lastStatus == job_model.JobStatusEnumRunning {
		s.SetJobStatus(job.ID, job_model.JobStatusEnumRunning)
		return true
	}

	// problems
	jobLogger.Error().Str("job_id", job.ID.String()).Msg("Job is not healthy")

	return false
	// TODO: do something about it
	// - delete this job
	// - tell the deployment to create a new job
	// - avoid placing the same job on the same machine
}

func (s *JobService) GetJobs() ([]job_model.Job, error) {
	var jobs []job_model.Job
	err := s.db.Gorm.Find(&job_model.Job{}).Where("accepted_at IS NOT NULL AND status != ?", job_model.JobStatusEnumStopped).Find(&jobs).Error
	return jobs, err
}

func (s *JobService) GetJob(jobID uuid.UUID) (job_model.Job, error) {
	var job job_model.Job
	err := s.db.Gorm.First(&job, jobID).Error
	return job, err
}

func (s *JobService) GetJobsByServiceID(serviceID uuid.UUID) ([]job_model.Job, error) {
	var jobs []job_model.Job
	err := s.db.Gorm.Find(&job_model.Job{}).Where("service_id = ?", serviceID).Find(&jobs).Error
	return jobs, err
}

// check on the rapid ticker frequency if all the watched jobs are healthy
// this is the main loop, multi tenant. it will inform the deployment workflow is something is not healthy
// healty = having a healthy status in redis
func (s *JobService) RunHealthLoop() {
	for range s.ticker.RapidTicker.C {
		jobs, err := s.GetJobs()
		if err != nil {
			jobLogger.Fatal().Err(err).Msg("Failed to get the running jobs to check their health")
			continue
		}
		wg := sync.WaitGroup{}

		healthyJobs := []job_model.Job{}
		unhealthyJobs := []job_model.Job{}

		// run all the jobs in parallel
		for _, job := range jobs {
			wg.Add(1)
			go func(job job_model.Job) {
				defer wg.Done()
				healthy := s.WatchJob(job)
				if healthy {
					healthyJobs = append(healthyJobs, job)
				} else {
					unhealthyJobs = append(unhealthyJobs, job)
				}
			}(job)
		}

		wg.Wait()

		go s.unhandleUnhealthyJobs(unhealthyJobs)
	}
}

// this will update the job status and send a signal to the deployment workflow
func (s *JobService) unhandleUnhealthyJobs(jobs []job_model.Job) {
	contextWithTimeout, _ := context.WithTimeout(context.Background(), 10*time.Second)

	for _, job := range jobs {
		// send a message to the deployment workflow
		s.temporalClient.UpdateWorkflow(contextWithTimeout, client.UpdateWorkflowOptions{
			WorkflowID:   job.DeploymentWorkflowID,
			RunID:        job.DeploymentRunID,
			UpdateName:   temporal_constants.UnhealthyJobSignal,
			WaitForStage: client.WorkflowUpdateStageAccepted,
			Args:         []interface{}{},
		})
	}
}
