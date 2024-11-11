package service

import (
	builder_model "brume.dev/builder/model"
	"brume.dev/internal/db"
	"brume.dev/runner"
	runner_model "brume.dev/runner/model"
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

func (s *ServiceService) UpdateBuilder(serviceId uuid.UUID, data builder_model.BuilderData) (*builder_model.Builder, error) {
	builder := &builder_model.Builder{
		ServiceId: serviceId,
		Type:      "generic-docker",
		Data:      data,
	}

	err := s.db.Gorm.Save(builder).Error

	return builder, err
}

func (s *ServiceService) UpdateRunner(serviceId uuid.UUID, data runner_model.RunnerData) (*runner_model.Runner, error) {

	runner := &runner_model.Runner{
		ServiceId: serviceId,
		Type:      "generic-docker",
		Data:      data,
	}

	err := s.db.Gorm.Save(runner).Error

	return runner, err
}

func (s *ServiceService) GetService(serviceId uuid.UUID) (*service_model.Service, error) {
	service := &service_model.Service{}
	err := s.db.Gorm.First(service, serviceId).Error
	return service, err
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
