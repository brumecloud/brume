package log_model

import (
	"time"

	"github.com/google/uuid"
)

type Log struct {
	ProjectID      uuid.UUID `gorm:"type:UUID;index"`
	ServiceID      uuid.UUID `gorm:"type:UUID;index"`
	DeploymentID   uuid.UUID `gorm:"type:UUID;index"`
	DeploymentName string

	ID        uuid.UUID
	Message   string
	Level     string
	Timestamp time.Time
}

// this is the format of the logs that the agent sends to the orchestrator
type AgentLogs struct {
	JobID     string `json:"job_id" validate:"required,uuid"`
	Message   string `json:"message" validate:"required"`
	Level     string `json:"level" validate:"required"`
	Timestamp string `json:"timestamp" validate:"required"`
}
