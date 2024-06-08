package client_v1

import (
	"context"

	gen "brume.dev/internal/gen/brume/v1"
)

// Authentificate with Brume server using email and password
// Returns a token if the login is successful
func (c *BrumeClient) PasswordLogin(email string, password string) (string, error) {
	res, err := c.authn.PasswordLogin(context.Background(), &gen.LoginRequest{
		Email:    email,
		Password: password,
	})

	if err != nil {
		return "", err
	}

	return res.Token, nil
}
