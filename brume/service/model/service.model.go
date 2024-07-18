package service_model

import (
	executor_model "brume.dev/executor/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Service struct {
	gorm.Model
	ID       uuid.UUID `gorm:"type:uuid;primaryKey"`
	Name     string
	Executor *executor_model.Executor `gorm:"foreignKey:ServiceId;references:ID"`

	ProjectID uuid.UUID
}

type BuilderJSON struct {
	Type     string
	Artifact string
}

type ExecutorJSON struct {
	Type     string
	Artifact string
}
