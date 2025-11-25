package temporal_client

//!! This file only create and connect the orchestrator to temporal
//!! All the worker configuration is done in the durable module

import (
	"fmt"
	"log/slog"

	"brume.dev/internal/config"
	"brume.dev/internal/log"
	"github.com/rs/zerolog"
	slogzerolog "github.com/samber/slog-zerolog/v2"
	"go.temporal.io/sdk/client"
	"go.uber.org/fx"
)

var logger = log.GetLogger("internal.durable.client.temporal-client")

func zerologToSlogLevel(level zerolog.Level) slog.Level {
	switch level {
	case zerolog.DebugLevel:
		return slog.LevelDebug
	case zerolog.InfoLevel:
		return slog.LevelInfo
	case zerolog.WarnLevel:
		return slog.LevelWarn
	case zerolog.ErrorLevel:
		return slog.LevelError
	case zerolog.FatalLevel:
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}

type TemporalClient struct {
	Client           client.Client
	DefaultTaskQueue string
	config           *config.BrumeConfig
}

var TemporalClientModule = fx.Module("temporal_client", fx.Provide(NewTemporalClient))

var DefaultTaskQueue = "brume"

func NewTemporalClient(lc fx.Lifecycle, cfg *config.BrumeConfig) *TemporalClient {

	slog := slog.New(slogzerolog.Option{
		Logger: &logger,
		Level:  zerologToSlogLevel(logger.GetLevel()),
	}.NewZerologHandler())

	client, err := client.NewLazyClient(client.Options{
		Logger:    slog,
		HostPort:  fmt.Sprintf("%s:%d", cfg.DurableConfig.TemporalHost, cfg.DurableConfig.TemporalPort),
		Namespace: cfg.DurableConfig.TemporalNamespace,
	})

	if err != nil {
		logger.Error().Err(err).Msg("Failed to create temporal client")
		return nil
	}

	temporalClient := &TemporalClient{
		DefaultTaskQueue: DefaultTaskQueue,
		config:           cfg,
		Client:           client,
	}

	return temporalClient
}
