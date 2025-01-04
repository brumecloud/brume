package temporal

import (
	"fmt"

	config "brume.dev/internal/config"
	brume_log "brume.dev/internal/log"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/client"
)

var logger = log.With().Str("module", "temporal").Logger()

func NewClient(cfg *config.BrumeConfig) client.Client {
	dsn := fmt.Sprintf("%s:%d", cfg.TemporalHost, cfg.TemporalPort)

	logger.Info().Str("dsn", dsn).Msg("Initializing Temporal client")

	c, err := client.Dial(client.Options{
		HostPort: dsn,
		Logger:   brume_log.NewTemporalZeroLogger(logger),
	})
	if err != nil {
		logger.Error().Err(err).Msg("Failed to dial Temporal client")
		panic(err)
	}

	logger.Info().Msg("Connected to Temporal server")

	return c
}
