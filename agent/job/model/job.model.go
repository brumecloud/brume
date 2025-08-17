package job_model

import (
	"time"
)

type JobType string

const (
	// run a service
	DockerRunningJob JobType = "runner"

	// create artifact for a service
	BuilderRunningJob JobType = "builder"

	// deploy a cloud stack, where a runner runs
	StackRunningJob   JobType = "stack"
)

type RunningJob struct {
	ID           string `gorm:"primaryKey"`
	JobType      JobType

	DeploymentID string
	ServiceID    string

	ContainerID *string

	LastCheckAt time.Time
	CreatedAt time.Time
}
