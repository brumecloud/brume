package runner_model

import (
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Runner struct {
	gorm.Model
	ID        uuid.UUID `gorm:"type:uuid;primaryKey"`
	ServiceId uuid.UUID
	Name      string
	Type      string

	Image string
}
