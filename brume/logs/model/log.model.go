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
