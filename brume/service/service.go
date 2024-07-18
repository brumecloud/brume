package service

import (
	"brume.dev/executor"
	"brume.dev/internal/db"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
)

type ServiceService struct {
	db              *db.DB
	executorService *executor.ExecutorService
}

func NewServiceService(db *db.DB, executorService *executor.ExecutorService) *ServiceService {
	return &ServiceService{
		db:              db,
		executorService: executorService,
	}
}

func (s *ServiceService) CreateService(name string, image string) (*service_model.Service, error) {
	id, _ := uuid.NewRandom()

	exec, execErr := s.executorService.CreateDockerExecutor(image)

	if execErr != nil {
		return nil, execErr
	}

	service := &service_model.Service{
		Name:     name,
		ID:       id,
		Executor: exec,
	}

	err := s.db.Gorm.Create(service).Error

	return service, err
}
