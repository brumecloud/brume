package job_service

import (
	"context"
	"fmt"
	"time"

	job_model "brume.dev/jobs/model"
	"github.com/redis/go-redis/v9"
)

type JobService struct {
	redisClient *redis.Client
}

func NewJobService(redisClient *redis.Client) *JobService {
	return &JobService{redisClient: redisClient}
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
