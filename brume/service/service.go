package service

import (
	"brume.dev/internal/db"
	"brume.dev/runner"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
)

type ServiceService struct {
	db            *db.DB
	runnerService *runner.RunnerService
}

func NewServiceService(db *db.DB, runnerService *runner.RunnerService) *ServiceService {
	return &ServiceService{
		db:            db,
		runnerService: runnerService,
	}
}

func (s *ServiceService) CreateService(name string, projectId uuid.UUID, image string) (*service_model.Service, error) {
	id, _ := uuid.NewRandom()

	service := &service_model.Service{
		Name:      name,
		ID:        id,
		ProjectID: projectId,
	}

	err := s.db.Gorm.Create(service).Error

	if err != nil {
		return nil, err
	}

	runner, execErr := s.runnerService.CreateDockerExecutor(image, id)

	if execErr != nil {
		return nil, execErr
	}

	service.Runner = runner
	s.db.Gorm.Save(service)

	return service, err
}
