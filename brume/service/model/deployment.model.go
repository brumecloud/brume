package service_model

import (
	"time"

	builder_model "brume.dev/builder/model"
	runner_model "brume.dev/runner/model"
	"github.com/google/uuid"
)

type Deployment struct {
	ID uuid.UUID `gorm:"type:uuid;primaryKey"`

	Source    DeploymentSource `gorm:"type:jsonb"`
	DeployLog DeploymentLog    `gorm:"type:jsonb"`

	Env string

	RunnerData  runner_model.RunnerData   `gorm:"type:jsonb"`
	BuilderData builder_model.BuilderData `gorm:"type:jsonb"`

	AuthorID uuid.UUID

	CreatedAt time.Time
}

type DeploymentSourceType string

const (
	DeploymentSourceTypeBranch DeploymentSourceType = "branch"
	DeploymentSourceTypeTag    DeploymentSourceType = "console"
)

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
	ID uuid.UUID `gorm:"type:uuid;primaryKey"`

	Status   DeploymentStatus
	Duration time.Duration
	Date     time.Time
}
