package user

import (
	user "brume.dev/account/user/model"
	"brume.dev/internal/db"
	project "brume.dev/project/model"
	"gorm.io/gorm"
)

type UserService struct {
	db *db.DB
}

func NewUserService(db *db.DB) *UserService {
	return &UserService{
		db: db,
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

func (s *UserService) GetUserProjects(user *user.User) (*user.User, error) {
	err := s.db.Gorm.Preload("Projects", func(db *gorm.DB) *gorm.DB {
		return db.Order("created_at DESC")
	}).First(&user, "id = ?", user.ID).Error

	return user, err
}

func (s *UserService) AddUserProject(user *user.User, project *project.Project) (*user.User, error) {
	user.Projects = append(user.Projects, project)
	err := s.db.Gorm.Save(user).Error

	return user, err
}
