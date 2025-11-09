package deployment_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

	job_model "brume.dev/jobs/model"

	"github.com/google/uuid"
)

type Deployment struct {
	ID        uuid.UUID `gorm:"type:uuid;primaryKey"`
	ServiceID uuid.UUID `gorm:"type:uuid"`

	Name string
	Env  string

	ProjectID uuid.UUID `gorm:"type:uuid"`

	Source DeploymentSource `gorm:"type:jsonb"`

	Execution ExecutionData `gorm:"type:jsonb"`

	Jobs []*job_model.Job `gorm:"foreignKey:DeploymentID"`

	CreatedAt time.Time
}

type ExecutionData struct {
	ContainerID string
	LastLogs    time.Time
}

func (e *ExecutionData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &e)
}

func (e *ExecutionData) Value() (driver.Value, error) {
	return json.Marshal(e)
}

type DeploymentSourceType string

const (
	DeploymentSourceTypeGit     DeploymentSourceType = "git"
	DeploymentSourceTypeConsole DeploymentSourceType = "console"
)

type DeploymentSource struct {
	// if console everything is empty
	Type DeploymentSourceType

	Branch  *string
	Commit  *string
	Message *string
}

func (d *DeploymentSource) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &d)
}

func (d *DeploymentSource) Value() (driver.Value, error) {
	return json.Marshal(d)
}

type DeploymentStatus string

const (
	DeploymentStatusPending DeploymentStatus = "pending"
	DeploymentStatusSuccess DeploymentStatus = "success"
	DeploymentStatusFailed  DeploymentStatus = "failed"
)

type DeploymentLog struct {
	Status   DeploymentStatus
	Duration time.Duration
	Date     time.Time
}

func (d *DeploymentLog) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &d)
}

func (d *DeploymentLog) Value() (driver.Value, error) {
	return json.Marshal(d)
}
