package server

import (
	"context"

	"brume.dev/account/org"
	"brume.dev/account/user/model"
	"brume.dev/internal/db"
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

func (s *OrganizationServer) GetUserOrganizations(ctx context.Context, _ *v1.Empty) (*v1.ListOrganization, error) {
	userEmail := ctx.Value("user")

	var user *user.User
	err := s.db.Gorm.First(&user, "email = ?", userEmail).Error

	log.Debug().Str("email", user.Email).Msg("get user")

	if err != nil {
		return nil, err
	}

	var orgs []*org.Organization
	err = s.db.Gorm.Find(&orgs, "id = ?", user.OrganizationID).Error

	if err != nil {
		return nil, err
	}

	orgsV1 := make([]*v1.Organization, len(orgs))

	for i, org := range orgs {
		orgsV1[i] = &v1.Organization{
			Name: org.Name,
			Id:   org.ID.String(),
		}
	}

	resp := &v1.ListOrganization{
		Organizations: orgsV1,
	}

	return resp, nil
}
