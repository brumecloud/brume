package runner_interfaces

import (
	"context"
	"time"

	log_model "brume.dev/logs/model"
	service_model "brume.dev/service/model"
)

// ContainerRunner is the interface to interact for all runners running OCI images
type ContainerRunner interface {
	// Start the container using the service definition
	StartService(ctx context.Context, deployment *service_model.Deployment) (string, error)

	// Stop the container
	StopService(ctx context.Context, deployment *service_model.Deployment) error

	// Get the log in Brume Log Format
	GetLogs(ctx context.Context, deployment *service_model.Deployment) ([]*log_model.Log, time.Time, error)
}
