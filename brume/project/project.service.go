package project

import (
	"brume.dev/internal/db"
	project "brume.dev/project/model"
	"brume.dev/service"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

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

func (s *ProjectService) GetProjectByID(id uuid.UUID) (*project.Project, error) {
	var project *project.Project

	err := s.db.Gorm.First(&project, "id = ?", id).Error

	if err != nil {
		return nil, err
	}

	return project, nil
}

func (s *ProjectService) DeployProject(projectId uuid.UUID) (*project.Project, error) {
	project, err := s.GetProjectByID(projectId)

	for _, service := range project.Services {
		err = s.ServiceService.DeployService(service.ID)
	}

	if err != nil {
		return nil, err
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
		return db.Preload("Builder").Preload("Runner").Order("created_at DESC")
	}).First(&project, "id = ?", project.ID).Error

	return project, err
}

func (s *ProjectService) AddServiceToProject(project *project.Project, service *service_model.Service) (*project.Project, error) {
	project.Services = append(project.Services, service)
	err := s.db.Gorm.Save(project).Error

	return project, err
}
