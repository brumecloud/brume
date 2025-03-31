package temporal

import (
	"fmt"

	config "brume.dev/internal/config"
	"brume.dev/internal/log"
	"go.temporal.io/sdk/client"
)

var logger = log.GetLogger("temporal_client")

func NewClient(cfg *config.BrumeConfig) client.Client {
	dsn := fmt.Sprintf("%s:%d", cfg.TemporalConfig.Host, cfg.TemporalConfig.Port)

	logger.Info().Str("dsn", dsn).Msg("Initializing Temporal client")

	c, err := client.Dial(client.Options{
		HostPort: dsn,
		Logger:   log.NewTemporalZeroLogger(logger),
	})
	if err != nil {
		logger.Error().Err(err).Msg("Failed to dial Temporal client")
		panic(err)
	}

	logger.Info().Msg("Connected to Temporal server")

	return c
}
