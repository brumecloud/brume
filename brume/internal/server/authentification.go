package server

import (
	"context"

	v1 "github.com/brume/brume/internal/gen/brume/v1"
	"github.com/rs/zerolog/log"
)

type AuthentificationServer struct {
	v1.UnimplementedAuthentificationServer
}

func NewAuthentificationServer() *AuthentificationServer {
	return &AuthentificationServer{}
}

func (s *AuthentificationServer) PasswordLogin(ctx context.Context, req *v1.LoginRequest) (*v1.LoginResponse, error) {
	log.Debug().Str("Email", req.GetEmail()).Str("Password", req.GetPassword()).Msg("Login attempt")

	return &v1.LoginResponse{
		Token: "ok",
	}, nil
}
