package client_v1

import (
	"context"

	gen "brume.dev/internal/gen/brume/v1"
	"google.golang.org/grpc/metadata"
)

func (c *BrumeClient) GetUserOrganizations(token string) ([]*gen.Organization, error) {
	md := metadata.Pairs("authorization", "Bearer "+token)
	ctx := metadata.NewOutgoingContext(context.Background(), md)

	reqs, err := c.orgs.GetUserOrganizations(ctx, &gen.Empty{})

	return reqs.Organizations, err
}
