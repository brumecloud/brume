package server

import (
	"context"
	"errors"

	"github.com/brume/brume/account/user"
	"github.com/brume/brume/internal/db"
	v1 "github.com/brume/brume/internal/gen/brume/v1"
	"github.com/rs/zerolog/log"
)

type AuthentificationServer struct {
	v1.UnimplementedAuthentificationServer

	db *db.DB
}

func NewAuthentificationServer(db *db.DB) *AuthentificationServer {
	return &AuthentificationServer{
		db: db,
	}
}

func (s *AuthentificationServer) PasswordLogin(ctx context.Context, req *v1.LoginRequest) (*v1.LoginResponse, error) {
	log.Debug().Str("Email", req.GetEmail()).Str("Password", req.GetPassword()).Msg("Login attempt")

	var user user.User
	err := s.db.Gorm.First(&user, "email = ?", req.GetEmail()).Error

	if err != nil {
		log.Debug().Err(err).Str("email", req.GetEmail()).Msg("Failed to find user")
		return nil, errors.New("BRU-AUTH-1, Invalid credentials")
	}

	if err := user.CheckPassword(req.GetPassword()); err != nil {
		log.Debug().Err(err).Msg("Failed to authenticate user")
		return nil, errors.New("BRU-AUTH-2, Invalid credentials")
	}

	tokenString, tokenErr := NewToken(&user)

	if tokenErr != nil {
		log.Debug().Err(tokenErr).Msg("Failed to generate token")
		return nil, errors.New("BRU-AUTH-3, Failed to generate token")
	}

	return &v1.LoginResponse{
		Token: tokenString,
	}, nil
}
