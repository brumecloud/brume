package project_model

import (
	"time"

	service_model "brume.dev/service/model"
	"github.com/google/uuid"

	// "gorm.io/datatypes"
	"gorm.io/gorm"
)

type Project struct {
	gorm.Model

	ID          uuid.UUID
	Name        string
	Description string

	Services []*service_model.Service `gorm:"foreignKey:ProjectID;references:ID"`
}

type ProjectVariables struct {
	Name  string
	Value string
	Tags  []string
}

type ProjectEvent struct {
	gorm.Model
	ID        uuid.UUID `gorm:"type:uuid;primaryKey"`
	ProjectID uuid.UUID `gorm:"type:uuid"`
	Timestamp time.Time `gorm:"type:timestamp;default:CURRENT_TIMESTAMP"`
	Type      string
	Data      string
}
