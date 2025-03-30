package service_model

import (
	builder_model "brume.dev/builder/model"
	deployment_model "brume.dev/deployment/model"
	job_model "brume.dev/jobs/model"
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
	LiveRunner  *runner_model.Runner   `gorm:"foreignKey:LiveRunnerID"`
	LiveBuilder *builder_model.Builder `gorm:"foreignKey:LiveBuilderID"`

	LiveRunnerID  *uuid.UUID
	LiveBuilderID *uuid.UUID

	// model which is used for real time interactions
	DraftRunner  *runner_model.Runner   `gorm:"foreignKey:DraftRunnerID"`
	DraftBuilder *builder_model.Builder `gorm:"foreignKey:DraftBuilderID"`

	DraftRunnerID  *uuid.UUID
	DraftBuilderID *uuid.UUID

	Deployments []*deployment_model.Deployment `gorm:"foreignKey:ServiceID"`
	Jobs        []*job_model.Job               `gorm:"foreignKey:ServiceID"`

	ProjectID uuid.UUID
}
