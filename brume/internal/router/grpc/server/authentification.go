package grpc_server

import (
	"context"

	"brume.dev/internal/common"
	v1 "brume.dev/internal/gen/brume/v1"
	"brume.dev/internal/log"
)

var logger = log.GetLogger("grpc_server")

type GRPCAuthentificationServer struct {
	v1.AuthentificationServer
	authService *common.AuthentificationService
}

func NewGRPCAuthentificationServer(s *common.AuthentificationService) *GRPCAuthentificationServer {
	return &GRPCAuthentificationServer{
		authService: s,
	}
}

func (s *GRPCAuthentificationServer) PasswordLogin(ctx context.Context, req *v1.LoginRequest) (*v1.LoginResponse, error) {
	logger.Debug().Str("Email", req.GetEmail()).Str("Password", req.GetPassword()).Msg("Login attempt")

	tokenString, err := s.authService.PasswordLogin(req.GetEmail(), req.GetPassword())
	if err != nil {
		return nil, err
	}

	return &v1.LoginResponse{
		Token: tokenString,
	}, nil
}
