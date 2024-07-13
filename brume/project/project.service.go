package project

import (
	"brume.dev/internal/db"
	project "brume.dev/project/model"
	"github.com/google/uuid"
)

type ProjectService struct {
	db *db.DB
}

func NewProjectService(db *db.DB) *ProjectService {
	return &ProjectService{
		db: db,
	}
}

func (s *ProjectService) GetProjectByID(id string) (*project.Project, error) {
	var project *project.Project

	err := s.db.Gorm.First(&project, "id = ?", id).Error

	if err != nil {
		return nil, err
	}

	return project, nil
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
