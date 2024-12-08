package service_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

	builder_model "brume.dev/builder/model"
	runner_model "brume.dev/runner/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

/**
Service data structure
Get the data from draft elements first (there are the working elements)
if the model are empty, get the data from the runner and builder
This are the last deployed version. They only change when the user clicks on deploy
**/

type Service struct {
	gorm.Model
	ID   uuid.UUID `gorm:"type:uuid;primaryKey"`
	Name string

	// model which is used for deployment
	LiveRunner  *runner_model.Runner   `gorm:"foreignKey:LiveRunnerID"`
	LiveBuilder *builder_model.Builder `gorm:"foreignKey:LiveBuilderID"`

	LiveRunnerID  *uuid.UUID
	LiveBuilderID *uuid.UUID

	// model which is used for real time interactions
	DraftRunner  *runner_model.Runner   `gorm:"foreignKey:DraftRunnerID"`
	DraftBuilder *builder_model.Builder `gorm:"foreignKey:DraftBuilderID"`

	DraftRunnerID  *uuid.UUID
	DraftBuilderID *uuid.UUID

	Deployments []*Deployment `gorm:"foreignKey:ServiceID"`

	ProjectID uuid.UUID
}
type Deployment struct {
	ID          uuid.UUID `gorm:"type:uuid;primaryKey"`
	ServiceID   uuid.UUID `gorm:"type:uuid"`
	ServiceName string

	ProjectID uuid.UUID `gorm:"type:uuid"`

	Source    DeploymentSource `gorm:"type:jsonb"`
	DeployLog DeploymentLog    `gorm:"type:jsonb"`

	Env string

	RunnerData  runner_model.RunnerData   `gorm:"type:jsonb"`
	BuilderData builder_model.BuilderData `gorm:"type:jsonb"`
	Execution   ExecutionData             `gorm:"type:jsonb"`

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

func (d *DeploymentSource) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &d)
}

func (d *DeploymentSource) Value() (driver.Value, error) {
	return json.Marshal(d)
}

type DeploymentSource struct {
	// if console everything is empty
	Type DeploymentSourceType

	Branch  *string
	Commit  *string
	Message *string
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
