package deployment

import (
	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/db"
	"github.com/google/uuid"
)

type DeploymentService struct {
	db *db.DB
}

func NewDeploymentService(db *db.DB) *DeploymentService {
	return &DeploymentService{db: db}
}

func (s *DeploymentService) GetDeployment(id uuid.UUID) (*deployment_model.Deployment, error) {
	var deployment deployment_model.Deployment
	err := s.db.Gorm.Where("id = ?", id).First(&deployment).Error
	return &deployment, err
}
