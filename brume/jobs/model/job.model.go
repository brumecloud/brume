package job_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

	deployment_model "brume.dev/deployment/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Job struct {
	gorm.Model

	ID          uuid.UUID `gorm:"type:uuid;primaryKey"`
	Status      JobStatusEnum
	ContainerID *string

	Price int

	CreatedAt  time.Time
	AcceptedAt *time.Time

	DeploymentWorkflowID string
	DeploymentRunID      string

	BidWorkflowID *string
	BidRunID      *string

	ServiceID uuid.UUID
	MachineID *uuid.UUID

	DeploymentID *uuid.UUID
	Deployment   *deployment_model.Deployment `gorm:"foreignKey:DeploymentID"`
}

type JobStatusEnum string

const (
	JobStatusEnumCreating  JobStatusEnum = "creating"
	JobStatusEnumPending   JobStatusEnum = "pending"
	JobStatusEnumRunning   JobStatusEnum = "running"
	JobStatusEnumStopped   JobStatusEnum = "stopped"
	JobStatusEnumUnhealthy JobStatusEnum = "unhealthy"
	JobStatusEnumFailed    JobStatusEnum = "failed"
)

type JobStatus struct {
	Status JobStatusEnum
	JobID  string
}

type JobMetadata struct {
	ContainerID string
}

func (j *JobStatus) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &j)
}

func (j *JobStatus) Value() (driver.Value, error) {
	return json.Marshal(j)
}
