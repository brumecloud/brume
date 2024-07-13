package project

import (
	"brume.dev/internal/db"
	project "brume.dev/project/model"
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
