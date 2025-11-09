package service

import (
	"encoding/json"

	builder_model "brume.dev/builder/model"
	builder_service "brume.dev/builder/service"
	deployment_service "brume.dev/deployment"
	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	job_model "brume.dev/jobs/model"
	"brume.dev/runner"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

var logger = log.GetLogger("service")

type ServiceService struct {
	db                *db.DB
	runnerService     *runner.RunnerService
	builderService    *builder_service.BuilderService
	deploymentService *deployment_service.DeploymentService
}

func NewServiceService(db *db.DB, runnerService *runner.RunnerService, builderService *builder_service.BuilderService, deploymentService *deployment_service.DeploymentService) *ServiceService {
	return &ServiceService{
		db:                db,
		runnerService:     runnerService,
		builderService:    builderService,
		deploymentService: deploymentService,
	}
}

// deploying a service mean creating a deployment out of the current service configuration
// and then, creating all the required jobs for this deployment
// at least the runner job, maybe a builder job if needed
func (s *ServiceService) DeployService(serviceId uuid.UUID, source deployment_model.DeploymentSource) error {
	// service, err := s.GetService(serviceId)
	// if err != nil {
	// 	return err
	// }

	// deployment := &deployment_model.Deployment{
	// 	ID:        uuid.New(),
	// 	ServiceID: serviceId,
	// 	ProjectID: service.ProjectID,
	// 	Name:      service.Name + "-" + time.Now().Format("20060102150405"),

	// 	Source:      source,
	// 	// BuilderData: service.LiveBuilder.Data,
	// 	RunnerData:  service.LiveRunner.Data,

	// 	CreatedAt: time.Now(),
	// }

	// add the deployment to the service
	// err = s.db.Gorm.Model(service).Association("Deployments").Append(deployment)
	// if err != nil {
	// 	return err
	// }

	// logger.Info().Str("deployment_id", deployment.ID.String()).Msg("Deployment created")

	// // start the deployment
	// // this will create the right jobs for the deployment
	// err = s.deploymentService.StartDeployment(deployment.ID)
	// if err != nil {
	// 	return err
	// }

	// logger.Info().Str("deployment_id", deployment.ID.String()).Msg("All jobs for the deployment are created")

	return nil
}

func (s *ServiceService) UpdateBuilder(serviceId uuid.UUID, data json.RawMessage) (*builder_model.Builder, error) {
	// var err error

	// service, err := s.GetService(serviceId)
	// if err != nil {
	// 	return nil, err
	// }

	// if service.DraftBuilder == nil {
	// 	draftBuilder, err := s.builderService.DuplicateBuilder(service.LiveBuilder.ID)
	// 	if err != nil {
	// 		return nil, err
	// 	}

	// 	service.DraftBuilder = draftBuilder
	// }

	// draftBuilder := &builder_model.Builder{
	// 	ID:        service.DraftBuilder.ID,
	// 	ServiceId: serviceId,
	// 	Type:      "generic-docker",
	// 	Data:      data,
	// }

	// err = s.db.Gorm.Save(draftBuilder).Error
	// if err != nil {
	// 	return nil, err
	// }

	// service.DraftBuilderID = &draftBuilder.ID

	// err = s.db.Gorm.Save(service).Error

	// return draftBuilder, err

	return nil, nil
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
	logger.Error().Str("job_id", job.ID.String()).Msg("Job is unhealthy")

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

// func (s *ServiceService) GetServiceDeployments(serviceId uuid.UUID) ([]*deployment_model.Deployment, error) {
// 	service, err := s.GetService(serviceId)
// 	if err != nil {
// 		return nil, err
// 	}

// 	// we dont want to load the deployments in the service object
// 	err = s.db.Gorm.Preload("Deployments").First(service, serviceId).Error

// 	return service.Deployments, err
// }

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

	// runner, execErr := s.runnerService.CreateDockerExecutor(service.ID)
	// builder, builderErr := s.builderService.CreateDockerBuilder(service.ID)

	// if execErr != nil || builderErr != nil {
	// 	return nil, execErr
	// }

	// service.DraftRunner = runner
	// service.DraftBuilder = builder

	s.db.Gorm.Save(service)

	return service, err
}

func (s *ServiceService) GetServiceJobs(serviceId uuid.UUID) ([]*job_model.Job, error) {
	jobs := []*job_model.Job{}
	err := s.db.Gorm.Where("service_id = ?", serviceId).Find(&jobs).Error
	return jobs, err
}
