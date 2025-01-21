package job_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

	deployment_model "brume.dev/deployment/model"
	"github.com/google/uuid"
)

type Job struct {
	ID uuid.UUID

	Price int

	CreatedAt  time.Time
	AcceptedAt *time.Time

	WorkflowID string
	RunID      string

	ServiceID  uuid.UUID
	MachineID  *uuid.UUID
	Deployment *deployment_model.Deployment
}

type JobStatusEnum string

const (
	JobStatusEnumRunning JobStatusEnum = "running"
	JobStatusEnumStopped JobStatusEnum = "stopped"
	JobStatusEnumFailed  JobStatusEnum = "failed"
)

type JobStatus struct {
	Status JobStatusEnum
	JobID  string
}

func (j *JobStatus) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &j)
}

func (j *JobStatus) Value() (driver.Value, error) {
	return json.Marshal(j)
}
