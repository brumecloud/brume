package service_model

import (
	runner_model "brume.dev/runner/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Service struct {
	gorm.Model
	ID     uuid.UUID `gorm:"type:uuid;primaryKey"`
	Name   string
	Runner *runner_model.Runner `gorm:"foreignKey:ServiceId;references:ID"`

	ProjectID uuid.UUID
}

type BuilderJSON struct {
	Type     string
	Artifact string
}

type RunnerJSON struct {
	Type     string
	Artifact string
}
