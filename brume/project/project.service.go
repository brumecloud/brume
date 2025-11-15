package project

import (
	org_model "brume.dev/account/org/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	project "brume.dev/project/model"
	"brume.dev/service"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
)

var logger = log.GetLogger("project.service")

type ProjectService struct {
	db             *db.DB
	ServiceService *service.ServiceService
}

func NewProjectService(db *db.DB, serviceService *service.ServiceService) *ProjectService {
	return &ProjectService{
		db:             db,
		ServiceService: serviceService,
	}
}

// launch the deploy of all the services of the projec (if needed)
func (s *ProjectService) DeployProject(projectId uuid.UUID) (*project.Project, error) {
	project, err := s.GetProjectByID(projectId)
	if err != nil {
		return nil, err
	}

	project, err = s.GetProject(project)
	if err != nil {
		return nil, err
	}

	logger.Info().Msgf("Deploying project %s", projectId)

	return s.GetProjectByID(projectId)
}

// check if one of the services is dirty (has some changes not yet applied)
func (s *ProjectService) IsDirty(project *project.Project) (bool, error) {
	var projectDirty bool

	project, err := s.GetProject(project)
	if err != nil {
		return false, err
	}

	return projectDirty, nil
}

func (s *ProjectService) GetProjectByID(id uuid.UUID) (*project.Project, error) {
	var project *project.Project

	err := s.db.Gorm.Preload("Services").First(&project, "id = ?", id).Error
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

	project, err = s.GetProject(project)
	if err != nil {
		return nil, err
	}

	return project, err
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

func (s *ProjectService) AssignProjectToOrganization(project *project.Project, organization *org_model.Organization) (*project.Project, error) {
	organization.Projects = append(organization.Projects, project)
	err := s.db.Gorm.Save(organization).Error

	return project, err
}

func (s *ProjectService) GetProject(project *project.Project) (*project.Project, error) {
	err := s.db.Gorm.Preload("Services").First(&project, "id = ?", project.ID).Error

	return project, err
}

func (s *ProjectService) AddServiceToProject(project *project.Project, service *service_model.Service) (*project.Project, error) {
	project.Services = append(project.Services, service)
	err := s.db.Gorm.Save(project).Error

	return project, err
}
