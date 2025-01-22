package job_service

import (
	"context"
	"fmt"
	"sync"
	"time"

	"brume.dev/internal/db"
	job_model "brume.dev/jobs/model"
	"github.com/brumecloud/agent/ticker"
	"github.com/redis/go-redis/v9"
	"go.uber.org/fx"
)

type JobService struct {
	redisClient *redis.Client
	ticker      *ticker.Ticker
	db          *db.DB
}

func NewJobService(lc fx.Lifecycle, redisClient *redis.Client, ticker *ticker.Ticker, db *db.DB) *JobService {
	js := &JobService{redisClient: redisClient, ticker: ticker, db: db}

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			logger.Info().Msg("Starting the job health loop")
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

// store the status of the job in redis
func (s *JobService) RecordJobStatus(jobID string, status job_model.JobStatusEnum) error {
	return s.redisClient.Set(context.Background(), fmt.Sprintf("job:%s:status", jobID), status, 10*time.Second).Err()
}

// get the status of the job from redis
func (s *JobService) GetJobStatus(jobID string) (job_model.JobStatusEnum, error) {
	status, err := s.redisClient.Get(context.Background(), fmt.Sprintf("job:%s:status", jobID)).Result()
	if err != nil {
		return job_model.JobStatusEnumFailed, err
	}
	return job_model.JobStatusEnum(status), nil
}

// do the actual job health check
func (s *JobService) WatchJob(job job_model.Job) {
	lastStatus, err := s.GetJobStatus(job.ID.String())
	if err != nil {
		logger.Error().Err(err).Msg("Failed to get the job status")
		return
	}

	if lastStatus == job_model.JobStatusEnumRunning {
		return
	}

	// problems
	logger.Error().Str("job_id", job.ID.String()).Msg("Job is not healthy")

	// TODO: do something about it
	// - delete this job
	// - tell the deployment to create a new job
	// - avoid placing the same job on the same machine
}

func (s *JobService) GetJobs() ([]job_model.Job, error) {
	var jobs []job_model.Job
	err := s.db.Gorm.Find(&job_model.Job{}).Where("accepted_at IS NOT NULL").Find(&jobs).Error
	return jobs, err
}

// check on the rapid ticker frequency if all the watched jobs are healthy
// healty = having a healthy status in redis
func (s *JobService) RunHealthLoop() {
	for range s.ticker.RapidTicker.C {
		jobs, err := s.GetJobs()
		if err != nil {
			logger.Fatal().Err(err).Msg("Failed to get the running jobs to check their health")
			continue
		}
		wg := sync.WaitGroup{}

		// run all the jobs in parallel
		for _, job := range jobs {
			wg.Add(1)
			go func(job job_model.Job) {
				defer wg.Done()
				s.WatchJob(job)
			}(job)
		}

		wg.Wait()
	}
}
