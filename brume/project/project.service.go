package project

import (
	"context"
	"encoding/json"
	"time"

	builder_model "brume.dev/builder/model"
	"brume.dev/internal/db"
	project "brume.dev/project/model"
	runner_model "brume.dev/runner/model"
	"brume.dev/service"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/client"
	"golang.org/x/exp/rand"
	"gorm.io/gorm"
)

type ProjectService struct {
	db             *db.DB
	ServiceService *service.ServiceService
	TemporalClient client.Client
}

func NewProjectService(db *db.DB, serviceService *service.ServiceService, temporalClient client.Client) *ProjectService {
	return &ProjectService{
		db:             db,
		ServiceService: serviceService,
		TemporalClient: temporalClient,
	}
}

func (s *ProjectService) IsDirty(project *project.Project) (bool, error) {
	var projectDirty bool

	project, err := s.GetProjectServices(project)

	if err != nil {
		return false, err
	}

	for _, service := range project.Services {
		if service.DraftRunner != nil || service.DraftBuilder != nil {
			return true, nil
		}
	}

	return projectDirty, nil
}

func (s *ProjectService) GetProjectByID(id uuid.UUID) (*project.Project, error) {
	var project *project.Project

	err := s.db.Gorm.First(&project, "id = ?", id).Error

	if err != nil {
		return nil, err
	}

	return project, nil
}

func (s *ProjectService) DeleteDraft(projectId uuid.UUID) (*project.Project, error) {
	project, err := s.GetProjectByID(projectId)

	if err != nil {
		return nil, err
	}

	project, err = s.GetProjectServices(project)

	if err != nil {
		return nil, err
	}

	for _, service := range project.Services {
		if service.DraftRunner != nil {
			err = s.db.Gorm.Delete(&runner_model.Runner{}, service.DraftRunnerID).Error

			if err != nil {
				return nil, err
			}
		}

		if service.DraftBuilder != nil {
			err = s.db.Gorm.Delete(&builder_model.Builder{}, service.DraftBuilderID).Error

			if err != nil {
				return nil, err
			}
		}
	}

	return project, err
}

func (s *ProjectService) DeployProject(projectId uuid.UUID) (*project.Project, error) {
	project, err := s.GetProjectByID(projectId)

	if err != nil {
		return nil, err
	}

	project, err = s.GetProjectServices(project)

	if err != nil {
		return nil, err
	}

	log.Info().Msgf("Deploying project %s", projectId)

	// move all the draft to non draft
	// when you deploy it gets save
	for _, service := range project.Services {
		if service.DraftRunnerID != nil {
			s.db.Gorm.Model(&service).Association("LiveRunner").Clear()
			s.db.Gorm.Model(&service).Association("LiveRunner").Append(service.DraftRunner)
			s.db.Gorm.Model(&service).Association("DraftRunner").Clear()
		}

		if service.DraftBuilderID != nil {
			s.db.Gorm.Model(&service).Association("LiveBuilder").Clear()
			s.db.Gorm.Model(&service).Association("LiveBuilder").Append(service.DraftBuilder)
			s.db.Gorm.Model(&service).Association("DraftBuilder").Clear()
		}

		workflowOpts := client.StartWorkflowOptions{
			TaskQueue: "node",
		}

		fullService, err := s.ServiceService.GetService(service.ID)

		if err != nil {
			return nil, err
		}

		we, err := s.TemporalClient.ExecuteWorkflow(context.Background(), workflowOpts, "RunServiceWorkflow", fullService)

		if err != nil {
			return nil, err
		}

		deployment := &service_model.Deployment{
			ID:        uuid.New(),
			ServiceID: service.ID,
			Source: service_model.DeploymentSource{
				Type: service_model.DeploymentSourceTypeConsole,
			},
			DeployLog: service_model.DeploymentLog{
				Status:   service_model.DeploymentStatusSuccess,
				Duration: time.Duration(rand.Intn(100)) * time.Second,
				Date:     time.Now(),
			},

			BuilderData: service.LiveBuilder.Data,
			RunnerData:  service.LiveRunner.Data,

			CreatedAt: time.Now(),
			Env:       "dev",
		}

		err = s.ServiceService.CreateDeployment(service.ID, deployment)

		if err != nil {
			log.Error().Msgf("Error creating deployment for service %s", service.ID)
		}

		log.Info().Msgf("Started service %s", we.GetID())
	}

	return s.GetProjectByID(projectId)
}

func (s *ProjectService) CreateProject(name string, description string) (*project.Project, error) {
	id, uuidErr := uuid.NewRandom()

	if uuidErr != nil {
		return nil, uuidErr
	}

	project := &project.Project{
		Name:        name,
		Description: description,
		ID:          id,
	}

	err := s.db.Gorm.Create(project).Error

	return project, err
}

func (s *ProjectService) GetProjectServices(project *project.Project) (*project.Project, error) {
	err := s.db.Gorm.Preload("Services", func(db *gorm.DB) *gorm.DB {
		return db.Preload("LiveBuilder").Preload("LiveRunner").Preload("DraftBuilder").Preload("DraftRunner").Order("created_at DESC")
	}).First(&project, "id = ?", project.ID).Error

	return project, err
}

func (s *ProjectService) AddServiceToProject(project *project.Project, service *service_model.Service) (*project.Project, error) {
	project.Services = append(project.Services, service)
	err := s.db.Gorm.Save(project).Error

	return project, err
}

func (s *ProjectService) PushEvent(projectId uuid.UUID, eventType string, eventData interface{}) error {
	eventDataJson, err := json.Marshal(eventData)

	if err != nil {
		return err
	}

	event := &project.ProjectEvent{
		ID:        uuid.New(),
		Timestamp: time.Now(),
		ProjectID: projectId,
		Type:      eventType,
		Data:      string(eventDataJson),
	}

	return s.db.Gorm.Save(event).Error
}
