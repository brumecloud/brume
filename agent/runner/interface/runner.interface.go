package runner_interfaces

import (
	"context"
	"time"

	deployment_model "brume.dev/deployment/model"
	log_model "brume.dev/logs/model"
)

type Runner interface {
	StartJob(ctx context.Context, deployment *deployment_model.Deployment) (string, error)
	StopJob(ctx context.Context, deployment *deployment_model.Deployment) error
	GetJobStatus(ctx context.Context, deployment *deployment_model.Deployment) (string, error)
	GetJobLogs(ctx context.Context, deployment *deployment_model.Deployment) ([]*log_model.Log, time.Time, error)
	GetRunnerHealth(ctx context.Context) (string, error)
}

type JobStatus string

const (
	JobStatusRunning    JobStatus = "running"
	JobStatusStopped    JobStatus = "stopped"
	AgentScrapingFailed JobStatus = "agent_scraping_failed"
)
