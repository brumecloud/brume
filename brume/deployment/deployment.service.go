package deployment

import (
	"time"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/db"
	job_model "brume.dev/jobs/model"
	"github.com/google/uuid"
)

type DeploymentService struct {
	db *db.DB
}

func NewDeploymentService(db *db.DB) *DeploymentService {
	return &DeploymentService{db: db}
}

// starting a deployment mean creating the right jobs for the deployments :
// - builder job if needed
// - runner job
func (s *DeploymentService) StartDeployment(id uuid.UUID) error {
	var deployment deployment_model.Deployment
	err := s.db.Gorm.Where("id = ?", id).First(&deployment).Error
	if err != nil {
		return err
	}

	builderJob := &job_model.Job{
		ID:        uuid.New(),
		JobType:   job_model.JobTypeBuilder,
		Status:    job_model.JobStatusEnumCreating,
		CreatedAt: time.Now(),
	}

	err = s.db.Gorm.Save(builderJob).Error
	if err != nil {
		return err
	}

	runnerJob := &job_model.Job{
		ID:        uuid.New(),
		JobType:   job_model.JobTypeRunner,
		Status:    job_model.JobStatusEnumBlocked,
		CreatedAt: time.Now(),

		// until the artifact are not built
		BlockedBy:   builderJob,
		BlockedByID: &builderJob.ID,
	}

	err = s.db.Gorm.Save(runnerJob).Error
	if err != nil {
		return err
	}

	return nil
}

func (s *DeploymentService) GetDeployment(id uuid.UUID) (*deployment_model.Deployment, error) {
	var deployment deployment_model.Deployment
	err := s.db.Gorm.Where("id = ?", id).First(&deployment).Error
	return &deployment, err
}
