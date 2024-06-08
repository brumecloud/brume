package project_model

import (
	service "brume.dev/service/model"
	"github.com/google/uuid"
	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type Project struct {
	gorm.Model
	ID               uuid.UUID `gorm:"type:uuid;primaryKey"`
	Name             string
	Description      string
	ProjectVariables datatypes.JSONType[ProjectVariables]

	Services []*service.Service `gorm:"foreignKey:ProjectID;references:ID"`
}

type ProjectVariables struct {
	Name  string
	Value string
	Tags  []string
}
