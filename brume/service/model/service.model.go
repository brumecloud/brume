package service_model

import (
	builder_model "brume.dev/builder/model"
	runner_model "brume.dev/runner/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

/**
Service data structure
Get the data from draft elements first (there are the working elements)
if the model are empty, get the data from the runner and builder
This are the last deployed version. They only change when the user clicks on deploy
**/

type Service struct {
	gorm.Model
	ID   uuid.UUID `gorm:"type:uuid;primaryKey"`
	Name string

	// model which is used for deployment
	Runner  runner_model.Runner   `gorm:"foreignKey:ID"`
	Builder builder_model.Builder `gorm:"foreignKey:ID"`

	// model which is used for real time interactions
	DraftRunner  *runner_model.Runner   `gorm:"foreignKey:ID"`
	DraftBuilder *builder_model.Builder `gorm:"foreignKey:ID"`

	ProjectID uuid.UUID
}
