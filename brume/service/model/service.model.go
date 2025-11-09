package service_model

import (
	"database/sql/driver"
	"encoding/json"

	builder_model "brume.dev/builder/model"
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

	Source *source_model.Source `gorm:"jsonb" json:"source"`

	// two versions of the service
	// the live service is the one that is currently running
	// the draft service is the one that is being edited
	Live *BaseService `gorm:"jsonb" json:"live_service"`
	Draft *BaseService `gorm:"jsonb" json:"draft_service"`

	// deployments of the service
	// Deployments []*deployment_model.Deployment `gorm:"foreignKey:ServiceID"`

	// project of the service
	ProjectID uuid.UUID
}

func (s *Service) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &s)
}

func (s *Service) Value() (driver.Value, error) {
	return json.Marshal(s)
}

func (s *BaseService) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &s)
}

func (s *BaseService) Value() (driver.Value, error) {
	return json.Marshal(s)
}