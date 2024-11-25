package service

import (
	builder_model "brume.dev/builder/model"
	builder_service "brume.dev/builder/service"
	"brume.dev/internal/db"
	"brume.dev/runner"
	runner_model "brume.dev/runner/model"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
)

type ServiceService struct {
	db             *db.DB
	runnerService  *runner.RunnerService
	builderService *builder_service.BuilderService
}

func NewServiceService(db *db.DB, runnerService *runner.RunnerService, builderService *builder_service.BuilderService) *ServiceService {
	return &ServiceService{
		db:             db,
		runnerService:  runnerService,
		builderService: builderService,
	}
}

func (s *ServiceService) UpdateBuilder(serviceId uuid.UUID, data builder_model.BuilderData) (*builder_model.Builder, error) {
	var err error

	service, err := s.GetService(serviceId)

	if err != nil {
		return nil, err
	}

	if service.DraftBuilder == nil {
		draftBuilder, err := s.builderService.DuplicateBuilder(service.LiveBuilder.ID)

		if err != nil {
			return nil, err
		}

		service.DraftBuilder = draftBuilder
	}

	draftBuilder := &builder_model.Builder{
		ID:        service.DraftBuilder.ID,
		ServiceId: serviceId,
		Type:      "generic-docker",
		Data:      data,
	}

	err = s.db.Gorm.Save(draftBuilder).Error

	if err != nil {
		return nil, err
	}

	service.DraftBuilderID = &draftBuilder.ID

	err = s.db.Gorm.Save(service).Error

	return draftBuilder, err
}

func (s *ServiceService) UpdateRunner(serviceId uuid.UUID, data runner_model.RunnerData) (*runner_model.Runner, error) {
	var err error

	service, err := s.GetService(serviceId)

	if err != nil {
		return nil, err
	}

	// we need to create a new draft of the runner
	if service.DraftRunner == nil {
		draftRunner, err := s.runnerService.DuplicateRunner(service.LiveRunner.ID)

		if err != nil {
			return nil, err
		}

		service.DraftRunner = draftRunner
	}

	// update the draft runner
	draftRunner := &runner_model.Runner{
		ID:        service.DraftRunner.ID,
		ServiceId: serviceId,
		Type:      "generic-docker",
		Data:      data,
	}

	err = s.db.Gorm.Save(draftRunner).Error

	if err != nil {
		return nil, err
	}

	service.DraftRunnerID = &draftRunner.ID

	err = s.db.Gorm.Save(service).Error

	if err != nil {
		return nil, err
	}

	return draftRunner, err
}

func (s *ServiceService) DeployService(serviceId uuid.UUID) error {
	service, err := s.GetService(serviceId)

	if err != nil {
		return err
	}

	// set the draft live
	service.LiveRunner = service.DraftRunner
	service.LiveBuilder = service.DraftBuilder
	service.DraftRunner = nil
	service.DraftBuilder = nil

	err = s.db.Gorm.Save(&service).Error

	if err != nil {
		return err
	}

	return nil
}

func (s *ServiceService) GetService(serviceId uuid.UUID) (*service_model.Service, error) {
	service := &service_model.Service{}
	err := s.db.Gorm.Preload("DraftRunner").Preload("DraftBuilder").Preload("Runner").Preload("Builder").First(service, serviceId).Error
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

	runner, execErr := s.runnerService.CreateDockerExecutor(service.ID)
	builder, builderErr := s.builderService.CreateDockerBuilder(service.ID)

	if execErr != nil || builderErr != nil {
		return nil, execErr
	}

	service.DraftRunner = runner
	service.DraftBuilder = builder

	s.db.Gorm.Save(service)

	return service, err
}
