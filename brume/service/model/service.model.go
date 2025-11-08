package service_model

import (
	builder_model "brume.dev/builder/model"
	deployment_model "brume.dev/deployment/model"
	runner_model "brume.dev/runner/model"
	source_model "brume.dev/source/model"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type BaseService struct {
	Source *source_model.Source `json:"source"`
	Runner *runner_model.Runner `json:"runner"`
	Builder *builder_model.Builder `json:"builder"`
}

/**
Service data structure
Get the data from draft elements first (there are the working elements)
if the model are empty, get the data from the runner and builder
This are the last deployed version. They only change when the user clicks on deploy
**/

type Service struct {
	gorm.Model
	ID   uuid.UUID `gorm:"type:uuid;primaryKey"`

	// general settings
	Name string

	// two versions of the service
	// the live service is the one that is currently running
	// the draft service is the one that is being edited
	LiveService *BaseService `gorm:"jsonb" json:"live_service"`
	DraftService *BaseService `gorm:"jsonb" json:"draft_service"`

	// deployments of the service
	Deployments []*deployment_model.Deployment `gorm:"foreignKey:ServiceID"`

	// project of the service
	ProjectID uuid.UUID
}
