package service_model

import (
	builder_model "brume.dev/builder/model"
	runner_model "brume.dev/runner/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Service struct {
	gorm.Model
	ID      uuid.UUID `gorm:"type:uuid;primaryKey"`
	Name    string
	Runner  runner_model.Runner   `gorm:"foreignKey:ServiceId"`
	Builder builder_model.Builder `gorm:"foreignKey:ServiceId"`

	ProjectID uuid.UUID
}
