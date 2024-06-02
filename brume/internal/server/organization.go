package server

import (
	"context"

	"github.com/brume/brume/internal/db"
	v1 "github.com/brume/brume/internal/gen/brume/v1"
	"github.com/rs/zerolog/log"
)

type OrganizationServer struct {
	v1.UnimplementedOrganizationServiceServer

	db *db.DB
}

func NewOrganizationServer(db *db.DB) *OrganizationServer {
	return &OrganizationServer{
		db: db,
	}
}

func (s *OrganizationServer) CreateOrganization(ctx context.Context, req *v1.CreateOrganizationRequest) (*v1.Organization, error) {
	return nil, nil
}

func (s *OrganizationServer) GetUserOrganizations(context.Context, *v1.Empty) (*v1.ListOrganization, error) {
}
