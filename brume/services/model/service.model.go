package services_model

import (
	"github.com/google/uuid"
	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type Service struct {
	gorm.Model
	ID       uuid.UUID `gorm:"type:uuid;primaryKey"`
	Name     string
	Builder  datatypes.JSONType[BuilderJSON]
	Executor datatypes.JSONType[ExecutorJSON]

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
