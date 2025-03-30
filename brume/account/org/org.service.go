package org

import (
	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	project_model "brume.dev/project/model"
)

var logger = log.GetLogger("account.org")

type OrganizationService struct {
	db *db.DB
}

func NewOrganizationService(db *db.DB) *OrganizationService {
	return &OrganizationService{
		db: db,
	}
}

func (s *OrganizationService) GetUserOrganization(email string) ([]*org.Organization, error) {
	logger.Trace().Str("email", email).Msg("Getting user organization")

	var user *user.User
	err := s.db.Gorm.First(&user, "email = ?", email).Error

	if err != nil {
		logger.Warn().Err(err).Str("email", email).Msg("Error getting user")
		return nil, err
	}

	var orgs []*org.Organization
	err = s.db.Gorm.Find(&orgs, "id = ?", user.OrganizationID).Error
	if err != nil {
		logger.Error().Err(err).Str("email", email).Msg("Error getting user organizations")
		return nil, err
	}

	return orgs, nil
}

func (s *OrganizationService) GetOrganizationProjects(org *org.Organization) ([]*project_model.Project, error) {
	logger.Trace().Str("org_id", org.ID.String()).Msg("Getting organization projects")

	err := s.db.Gorm.Preload("Projects").Find(&org).Error

	if err != nil {
		logger.Error().Err(err).Str("org_id", org.ID.String()).Msg("Error getting organization projects")
		return nil, err
	}

	return org.Projects, err
}
func (s *OrganizationService) AddProjectToOrganization(org *org.Organization, project *project_model.Project) error {
	logger.Trace().Str("org_id", org.ID.String()).Str("project_id", project.ID.String()).Msg("Adding project to organization")

	org.Projects = append(org.Projects, project)

	err := s.db.Gorm.Save(org).Error
	if err != nil {
		logger.Error().Err(err).Str("org_id", org.ID.String()).Str("project_id", project.ID.String()).Msg("Error adding project to organization")
		return err
	}

	return nil
}
