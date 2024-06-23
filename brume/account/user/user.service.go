package user

import (
	user "brume.dev/account/user/model"
	"brume.dev/internal/db"
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
