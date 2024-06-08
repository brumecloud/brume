package common

import (
	"errors"

	"brume.dev/account/user"
	"brume.dev/internal/db"
	"github.com/rs/zerolog/log"
)

type AuthentificationService struct {
	db *db.DB
}

func NewAuthentificationService(db *db.DB) *AuthentificationService {
	return &AuthentificationService{
		db: db,
	}
}

func (s *AuthentificationService) PasswordLogin(email string, password string) (string, error) {
	var user user.User
	err := s.db.Gorm.First(&user, "email = ?", email).Error

	if err != nil {
		log.Debug().Err(err).Str("email", email).Msg("Failed to find user")
		return "", errors.New("BRU-AUTH-1, Invalid credentials")
	}

	if err := user.CheckPassword(password); err != nil {
		log.Debug().Err(err).Msg("Failed to authenticate user")
		return "", errors.New("BRU-AUTH-2, Invalid credentials")
	}

	tokenString, tokenErr := NewToken(&user)

	if tokenErr != nil {
		log.Debug().Err(tokenErr).Msg("Failed to generate token")
		return "", errors.New("BRU-AUTH-3, Failed to generate token")
	}

	return tokenString, nil
}
