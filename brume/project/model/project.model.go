package project_model

import (
	service_model "brume.dev/service/model"

	// "gorm.io/datatypes"
	"gorm.io/gorm"
)

type Project struct {
	gorm.Model

	ID          string
	Name        string
	Description string

	Services []*service_model.Service `gorm:"foreignKey:ProjectID;references:ID"`
}
