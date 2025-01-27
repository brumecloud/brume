package service

import (
	"context"
	"time"

	builder_model "brume.dev/builder/model"
	builder_service "brume.dev/builder/service"
	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	temporal_constants "brume.dev/internal/temporal/constants"
	job_model "brume.dev/jobs/model"
	"brume.dev/runner"
	runner_model "brume.dev/runner/model"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
	"go.temporal.io/sdk/client"
	"gorm.io/gorm"
)

var logger = log.GetLogger("service")

type ServiceService struct {
	db             *db.DB
	runnerService  *runner.RunnerService
	builderService *builder_service.BuilderService
	temporalClient client.Client
}

func NewServiceService(db *db.DB, runnerService *runner.RunnerService, builderService *builder_service.BuilderService, temporalClient client.Client) *ServiceService {
	return &ServiceService{
		db:             db,
		runnerService:  runnerService,
		builderService: builderService,
		temporalClient: temporalClient,
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

func (s *ServiceService) DeleteService(serviceId uuid.UUID) (*service_model.Service, error) {
	service, err := s.GetService(serviceId)
	if err != nil {
		return nil, err
	}

	err = s.db.Gorm.Delete(service).Error
	if err != nil {
		logger.Error().Err(err).Msg("Failed to delete service")
		return nil, err
	}

	// find the running job associated with the service
	job := &job_model.Job{}
	err = s.db.Gorm.First(job, "service_id = ?", serviceId).Error
	if err != nil {
		// not found is ok, the service was never deployed
		if err == gorm.ErrRecordNotFound {
			return service, nil
		}

		logger.Error().Err(err).Msg("Failed to find running job")
		return nil, err
	}

	// this will stop the job, without any restart
	contextWithTimeout, _ := context.WithTimeout(context.Background(), 10*time.Second)

	s.temporalClient.UpdateWorkflow(contextWithTimeout, client.UpdateWorkflowOptions{
		WorkflowID:   job.DeploymentWorkflowID,
		RunID:        job.DeploymentRunID,
		UpdateName:   temporal_constants.StopJobSignal,
		WaitForStage: client.WorkflowUpdateStageAccepted,
		Args:         []interface{}{},
	})

	return service, nil
}

func (s *ServiceService) GetService(serviceId uuid.UUID) (*service_model.Service, error) {
	service := &service_model.Service{}
	err := s.db.Gorm.Preload("DraftRunner").Preload("DraftBuilder").Preload("LiveRunner").Preload("LiveBuilder").First(service, serviceId).Error
	return service, err
}

func (s *ServiceService) UpdateServiceSettings(serviceId uuid.UUID, name string) (*service_model.Service, error) {
	service, err := s.GetService(serviceId)
	if err != nil {
		return nil, err
	}

	service.Name = name

	return service, s.db.Gorm.Save(&service).Error
}

func (s *ServiceService) CreateDeployment(serviceId uuid.UUID, deployment *deployment_model.Deployment) error {
	service, err := s.GetService(serviceId)
	if err != nil {
		return err
	}

	err = s.db.Gorm.Model(service).Association("Deployments").Append(deployment)

	return err
}

func (s *ServiceService) GetServiceDeployments(serviceId uuid.UUID) ([]*deployment_model.Deployment, error) {
	service, err := s.GetService(serviceId)
	if err != nil {
		return nil, err
	}

	// we dont want to load the deployments in the service object
	err = s.db.Gorm.Preload("Deployments").First(service, serviceId).Error

	return service.Deployments, err
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
