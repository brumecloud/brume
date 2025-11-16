package job_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

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

	ID           string        `gorm:"type:varchar(255);primaryKey" json:"id"`
	JobType      JobType       `gorm:"type:varchar(255);not null" json:"job_type"`
	Status       JobStatusEnum `gorm:"type:varchar(255);not null" json:"status"`
	DeploymentID string        `gorm:"type:varchar(255);not null" json:"deployment_id"`

	CreatedAt  time.Time  `json:"created_at"`
	AcceptedAt *time.Time `json:"accepted_at"`

	// a runner job can be blocked by a builder job for example
	// the blocked job will be release when the blocking job is done
	BlockedBy   *Job    `gorm:"foreignKey:BlockedByID" json:"blocked_by"`
	BlockedByID *string `gorm:"type:varchar(255)" json:"blocked_by_id"`
}

type JobStatusEnum string

const (
	// init state
	JobStatusEnumCreating JobStatusEnum = "creating"
	// the job need to be picked by an agent (bidding process)
	JobStatusEnumPending JobStatusEnum = "pending"
	// the job is blocked by another job
	JobStatusEnumBlocked JobStatusEnum = "blocked"
	// the job is running on an agent & the orchestrator agrees
	JobStatusEnumRunning JobStatusEnum = "running"
	// the job is stopped by the orchestrator (mostly human intervention)
	JobStatusEnumStopped JobStatusEnum = "stopped"
	// the job is unhealthy (healthcheck failed)
	JobStatusEnumUnhealthy JobStatusEnum = "unhealthy"
	// all the healthchecks threshold are reached
	JobStatusEnumFailed JobStatusEnum = "failed"
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
