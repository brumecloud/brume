package user

import (
	org_service "brume.dev/account/org"
	user "brume.dev/account/user/model"
	"brume.dev/internal/db"
	project "brume.dev/project/model"
)

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
	var user *user.User

	err := s.db.Gorm.First(&user, "email = ?", email).Error

	if err != nil {
		return nil, err
	}

	return user, nil
}

func (s *UserService) GetUserProjects(user *user.User) ([]*project.Project, error) {
	orgs, err := s.orgService.GetUserOrganization(user.Email)

	if err != nil {
		return nil, err
	}

	org := orgs[0]

	projects, err := s.orgService.GetOrganizationProjects(org)

	if err != nil {
		return nil, err
	}

	// this is where we would add the authZ
	return projects, nil
}

func (s *UserService) AddUserProject(user *user.User, project *project.Project) (*user.User, error) {
	orgs, err := s.orgService.GetUserOrganization(user.Email)

	if err != nil {
		return nil, err
	}

	err = s.orgService.AddProjectToOrganization(orgs[0], project)

	return user, err
}
