package user

import (
	org_service "brume.dev/account/org"
	org_model "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	project "brume.dev/project/model"
)

var logger = log.GetLogger("account.user")

type UserService struct {
	db         *db.DB
	orgService *org_service.OrganizationService
}

func NewUserService(db *db.DB, orgService *org_service.OrganizationService) *UserService {
	return &UserService{
		db:         db,
		orgService: orgService,
	}
}

func (s *UserService) GetUserByEmail(email string) (*user.User, error) {
	logger.Trace().Str("email", email).Msg("Getting user by email")
	var user *user.User

	err := s.db.Gorm.First(&user, "email = ?", email).Error
	if err != nil {
		logger.Error().Err(err).Str("email", email).Msg("Error getting user by email")
		return nil, err
	}

	return user, nil
}

func (s *UserService) GetUserProjects(user *user.User) ([]*project.Project, error) {
	logger.Trace().Str("email", user.Email).Msg("Getting user projects")

	orgs, err := s.orgService.GetUserOrganization(user.Email)
	if err != nil {
		logger.Warn().Err(err).Str("email", user.Email).Msg("Error getting user organizations")
		return nil, err
	}

	org := orgs[0]

	projects, err := s.orgService.GetOrganizationProjects(org)
	if err != nil {
		logger.Warn().Err(err).Str("email", user.Email).Msg("Error getting user projects")
		return nil, err
	}

	// this is where we would add the authZ
	return projects, nil
}

func (s *UserService) AddUserProject(user *user.User, project *project.Project) (*user.User, error) {
	logger.Trace().Str("email", user.Email).Msg("Adding user project")

	orgs, err := s.orgService.GetUserOrganization(user.Email)
	if err != nil {
		logger.Warn().Err(err).Str("email", user.Email).Msg("Error getting user organizations")
		return nil, err
	}

	err = s.orgService.AddProjectToOrganization(orgs[0], project)
	if err != nil {
		logger.Warn().Err(err).Str("email", user.Email).Msg("Error adding project to organization")
		return nil, err
	}

	return user, err
}

func (s *UserService) GetUserOrganization(user *user.User) (*org_model.Organization, error) {
	logger.Trace().Str("email", user.Email).Msg("Getting user organization")

	var org *org_model.Organization

	err := s.db.Gorm.First(&org, "id = ?", user.OrganizationID).Error

	if err != nil {
		logger.Warn().Err(err).Str("email", user.Email).Msg("Error getting user organization")
		return nil, err
	}

	return org, nil
}
