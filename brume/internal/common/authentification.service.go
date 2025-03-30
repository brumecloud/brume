package common

import (
	"errors"

	user "brume.dev/account/user/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
)

var logger = log.GetLogger("authentification")

type AuthentificationService struct {
	db *db.DB
}

func NewAuthentificationService(db *db.DB) *AuthentificationService {
	return &AuthentificationService{
		db: db,
	}
}

func (s *AuthentificationService) PasswordLogin(email string, password string) (string, error) {
	logger.Trace().Str("email", email).Msg("Password login")

	var user user.User
	err := s.db.Gorm.First(&user, "email = ?", email).Error

	if err != nil {
		logger.Error().Err(err).Str("email", email).Msg("Failed to find user")
		return "", errors.New("BRU-AUTH-1, Invalid credentials")
	}

	if err := user.CheckPassword(password); err != nil {
		logger.Error().Err(err).Msg("Failed to authenticate user")
		return "", errors.New("BRU-AUTH-2, Invalid credentials")
	}

	tokenString, tokenErr := NewToken(&user)

	if tokenErr != nil {
		logger.Error().Err(tokenErr).Msg("Failed to generate token")
		return "", errors.New("BRU-AUTH-3, Failed to generate token")
	}

	return tokenString, nil
}
