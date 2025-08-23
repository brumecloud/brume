package brume_workos

import (
	"context"
	"errors"

	"brume.dev/internal/config"
	brume_log "brume.dev/internal/log"
	"github.com/lestrrat-go/jwx/v3/jwk"
	"github.com/lestrrat-go/jwx/v3/jwt"
	"github.com/workos/workos-go/v4/pkg/usermanagement"
	"go.uber.org/fx"
)

var logger = brume_log.GetLogger("workos")

type WorkOSClient struct {
	jwkSet jwk.Set
}

func NewWorkOSClient(lc fx.Lifecycle, cfg *config.BrumeConfig) *WorkOSClient {
	usermanagement.SetAPIKey(cfg.WorkOSConfig.ClientSecret)

	client := &WorkOSClient{}

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			jwksUrl, err := usermanagement.GetJWKSURL(cfg.WorkOSConfig.ClientID)
			if err != nil {
				logger.Panic().Err(err).Msg("Failed to get JWKS URL")
			}

			logger.Info().Str("jwks_url", jwksUrl.String()).Msg("Got JWKS URL")

			set, err := jwk.Fetch(ctx, jwksUrl.String())
			if err != nil {
				logger.Panic().Err(err).Msg("Failed to fetch JWKS")
			}

			logger.Info().Interface("jwkSet", set).Msg("JWK Set")

			client.jwkSet = set

			return nil
		},
	})

	return client
}

func (c *WorkOSClient) VerifyToken(accessToken string) (jwt.Token, error) {
	logger.Info().Str("accessToken", accessToken).Msg("Verifying token")

	token, err := jwt.Parse([]byte(accessToken), jwt.WithKeySet(c.jwkSet))
	if err != nil {
		logger.Error().Err(err).Msg("Failed to parse token")
		return nil, err
	}

	subject, ok := token.Subject()
	if !ok {
		logger.Error().Msg("Failed to get subject from token")
		return nil, errors.New("invalid token")
	}

	logger.Info().Str("subject", subject).Msg("Token valid")

	return token, nil
}

var WorkOSModule = fx.Module("workos", fx.Provide(NewWorkOSClient), fx.Invoke(func(workos *WorkOSClient) {}))
