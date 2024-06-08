package grpc_server

import (
	"context"
	"errors"

	org "brume.dev/account/org"
	v1 "brume.dev/internal/gen/brume/v1"
)

type OrganizationServer struct {
	v1.UnimplementedOrganizationServiceServer

	organizationService *org.OrganizationService
}

func NewOrganizationServer(s *org.OrganizationService) *OrganizationServer {
	return &OrganizationServer{
		organizationService: s,
	}
}

func (s *OrganizationServer) CreateOrganization(ctx context.Context, req *v1.CreateOrganizationRequest) (*v1.Organization, error) {
	return nil, nil
}

func (s *OrganizationServer) GetUserOrganizations(ctx context.Context, _ *v1.Empty) (*v1.ListOrganization, error) {
	userEmail := ctx.Value("user")

	if userEmail == "" {
		return nil, errors.New("user email not found")
	}
	email := string(userEmail.(string))

	orgs, err := s.organizationService.GetUserOrganization(email)

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
