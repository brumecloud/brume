package log_model

import (
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Log struct {
	gorm.Model
	ProjectID      uuid.UUID
	DeploymentID   uuid.UUID
	DeploymentName string

	ID        uuid.UUID
	Message   string
	Level     string
	Timestamp time.Time
}
