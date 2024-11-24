package temporal

import (
	brume_log "brume.dev/internal/log"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/client"
)

func NewClient() client.Client {
	logger := brume_log.GetLogger().With().Str("service", "temporal").Logger()

	c, err := client.Dial(client.Options{
		HostPort: "host.docker.internal:7233",
		Logger:   brume_log.NewTemporalZeroLogger(logger),
	})

	if err != nil {
		log.Error().Err(err).Msg("Failed to dial Temporal client")
		panic(err)
	}

	log.Info().Msg("Connected to Temporal server")

	return c
}
