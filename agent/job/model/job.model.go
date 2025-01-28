package job_model

import (
	"time"
)

type JobType string

const (
	DockerRunningJob JobType = "docker"
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
