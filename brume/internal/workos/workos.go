package brume_workos

import (
	"context"
	"errors"
	"net/http"

	"brume.dev/internal/config"
	brume_log "brume.dev/internal/log"
	"brume.dev/internal/router/http/cookies"
	"github.com/lestrrat-go/jwx/v3/jwk"
	"github.com/lestrrat-go/jwx/v3/jwt"
	"github.com/workos/workos-go/v4/pkg/usermanagement"
	"go.uber.org/fx"
)

var logger = brume_log.GetLogger("internal.workos")

type WorkOSClient struct {
	jwkSet jwk.Set
	cfg    *config.BrumeConfig
}

func NewWorkOSClient(lc fx.Lifecycle, cfg *config.BrumeConfig) *WorkOSClient {
	usermanagement.SetAPIKey(cfg.WorkOSConfig.ClientSecret)

	client := &WorkOSClient{
		cfg: cfg,
	}

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

func (c *WorkOSClient) VerifyTokenWithRefresh(accessToken string, refreshToken string, w http.ResponseWriter) (jwt.Token, error) {
	token, err := jwt.Parse([]byte(accessToken), jwt.WithKeySet(c.jwkSet), jwt.WithValidate(false))
	if err != nil {
		logger.Error().Err(err).Msg("Failed to parse token")
		return nil, err
	}

	subject, ok := token.Subject()
	if !ok {
		logger.Error().Msg("Failed to get subject from token")
		return nil, errors.New("invalid token")
	}

	if err = jwt.Validate(token); err != nil {
		// try refreshing the token
		resp, err := usermanagement.AuthenticateWithRefreshToken(context.Background(), usermanagement.AuthenticateWithRefreshTokenOpts{
			RefreshToken: refreshToken,
			ClientID:     c.cfg.WorkOSConfig.ClientID,
		})
		if err != nil {
			logger.Error().Err(err).Str("subject", subject).Msg("Failed to refresh token")
			return nil, err
		}

		logger.Info().Str("subject", subject).Msg("Refreshed token")

		// refresh the token
		cookies.GenerateUserCookie(resp.AccessToken, resp.RefreshToken, c.cfg, w)
	}

	return token, nil
}

var WorkOSModule = fx.Module("workos", fx.Provide(NewWorkOSClient), fx.Invoke(func(workos *WorkOSClient) {}))
