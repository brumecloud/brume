package runner_interfaces

import (
	"context"
	"time"

	deployment_model "brume.dev/deployment/model"
	job_model "brume.dev/jobs/model"
	log_model "brume.dev/logs/model"
)

type ContainerStatusResult struct {
	ContainerID string
	JobID       string
	Status      job_model.JobStatusEnum
}

// ContainerRunner is the interface to interact for all runners running OCI images
type ContainerRunner interface {
	// Start the container using the service definition
	StartService(ctx context.Context, deployment *deployment_model.Deployment) (string, error)

	// Stop the container
	StopService(ctx context.Context, deployment *deployment_model.Deployment) error

	// Get the log in Brume Log format
	GetLogs(ctx context.Context, deployment *deployment_model.Deployment) ([]*log_model.Log, time.Time, error)

	// Get the status of the container
	// true if the container is running and healthy, false otherwise
	GetStatus(ctx context.Context, deployment *deployment_model.Deployment) (bool, error)

	// Get the health of the container runner
	GetRunnerHealth(ctx context.Context) (bool, error)

	// Get all the running jobs
	GetAllRunningJobs() (map[string]ContainerStatusResult, error)
}
