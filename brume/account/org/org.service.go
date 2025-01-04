package org

import (
	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	project_model "brume.dev/project/model"
)

var logger = log.GetLogger("org")

type OrganizationService struct {
	db *db.DB
}

func NewOrganizationService(db *db.DB) *OrganizationService {
	return &OrganizationService{
		db: db,
	}
}

func (s *OrganizationService) GetUserOrganization(email string) ([]*org.Organization, error) {
	var user *user.User
	err := s.db.Gorm.First(&user, "email = ?", email).Error

	logger.Debug().Str("email", user.Email).Msg("get user")

	if err != nil {
		return nil, err
	}

	var orgs []*org.Organization
	err = s.db.Gorm.Find(&orgs, "id = ?", user.OrganizationID).Error
	if err != nil {
		return nil, err
	}

	return orgs, nil
}

func (s *OrganizationService) GetOrganizationProjects(org *org.Organization) ([]*project_model.Project, error) {
	err := s.db.Gorm.Preload("Projects").Find(&org).Error

	return org.Projects, err
}

func (s *OrganizationService) AddProjectToOrganization(org *org.Organization, project *project_model.Project) error {
	org.Projects = append(org.Projects, project)
	return s.db.Gorm.Save(org).Error
}
