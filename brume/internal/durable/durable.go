package durable

import (
	"context"
	"log/slog"
	"time"

	"brume.dev/internal/config"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	"github.com/dbos-inc/dbos-transact-golang/dbos"
	"github.com/rs/zerolog"
	slogzerolog "github.com/samber/slog-zerolog/v2"
	"go.uber.org/fx"
)

var DurableModule = fx.Module("durable", fx.Provide(NewDurable), fx.Invoke(func(d *Durable) {}))

var logger = log.GetLogger("durable")

type Durable struct {
	config *config.BrumeConfig
	dbos   *dbos.DBOSContext
}

// we need the db to create the durable database
func NewDurable(lc fx.Lifecycle, config *config.BrumeConfig, db *db.DB) *Durable {
	logger.Info().Msg("Initializing durable module")
	durable := &Durable{
		config: config,
	}
	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			durable.Start(ctx)
			return nil
		},
		OnStop: func(ctx context.Context) error {
			durable.Stop(ctx)
			return nil
		},
	})
	return durable
}

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

func (d *Durable) Start(ctx context.Context) {
	logger.Info().Str("app_name", d.config.DurableConfig.DurableName).Bool("admin_server", d.config.DurableConfig.AdminServer).Int("admin_server_port", d.config.DurableConfig.AdminServerPort).Msg("Starting durable executor")

	// convert the zerolog logger to a slog logger
	slog := slog.New(slogzerolog.Option{
		Logger: &logger,
		Level:  zerologToSlogLevel(logger.GetLevel()),
	}.NewZerologHandler())

	dbos, err := dbos.NewDBOSContext(ctx, dbos.Config{
		DatabaseURL:     d.config.DurableConfig.DatabaseConn + "/" + d.config.DurableConfig.DatabaseName,
		AppName:         d.config.DurableConfig.DurableName,
		AdminServer:     d.config.DurableConfig.AdminServer,
		AdminServerPort: d.config.DurableConfig.AdminServerPort,
		Logger:          slog,
	})

	if err != nil {
		logger.Error().Err(err).Msg("Failed to create DBOS context")
		return
	}

	logger.Info().Msg("DBOS context created")
	d.dbos = &dbos
	d.RegisterWorkflow()

	dbos.Launch()
	logger.Info().Msg("DBOS launched")
}

func (d *Durable) RegisterWorkflow() {
	logger.Info().Msg("Registering workflow")

	logger.Info().Msg("Workflow registered")
}

func (d *Durable) Stop(ctx context.Context) {
	dbos.Shutdown(*d.dbos, time.Second*2)
	logger.Info().Msg("DBOS shutdown completed")
}
