package job_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type JobType string

const (
	JobTypeRunner  JobType = "runner"
	JobTypeBuilder JobType = "builder"
	JobTypeStack   JobType = "stack"
)

// a job is a something that need to be executed by an agent
// runner jobs are the most common, they run user code
// builder jobs are used to build artifacts (like docker images, SPA artifacts, etc)
// stack jobs are used to deploy a cloud stack (like a kubernetes cluster, or an EC2)
type Job struct {
	gorm.Model

	ID      uuid.UUID     `gorm:"type:uuid;primaryKey"`
	JobType JobType       `gorm:"type:varchar(255);not null"`
	Status  JobStatusEnum `gorm:"type:varchar(255);not null"`

	CreatedAt  time.Time
	AcceptedAt *time.Time
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
