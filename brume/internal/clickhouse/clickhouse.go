package brume_clickhouse

import (
	"context"
	"crypto/tls"
	"fmt"

	"brume.dev/internal/config"
	brume_log "brume.dev/internal/log"
	"github.com/ClickHouse/clickhouse-go/v2"
	"github.com/ClickHouse/clickhouse-go/v2/lib/driver"
	"go.uber.org/fx"
)

var logger = brume_log.GetLogger("clickhouse")

type ClickhouseDB struct {
	Conn driver.Conn
}

func NewClickhouseDB(lc fx.Lifecycle, config *config.BrumeConfig) *ClickhouseDB {
	clickhouse := ClickhouseDB{}
	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			return clickhouse.Connect(ctx, config)
		},
		OnStop: func(ctx context.Context) error {
			return nil
		},
	})
	return &clickhouse
}

func (c *ClickhouseDB) Connect(ctx context.Context, config *config.BrumeConfig) error {
	logger.Debug().Str("host", config.ClickhouseConfig.Host).Int("port", config.ClickhouseConfig.Port).Msg("Connecting to clickhouse")
	logger.Debug().Str("db", config.ClickhouseConfig.DB).Str("user", config.ClickhouseConfig.User).Str("password", config.ClickhouseConfig.Password).Msg("Clickhouse credentials")
	conn, err := clickhouse.Open(&clickhouse.Options{
		Addr: []string{fmt.Sprintf("%s:%d", config.ClickhouseConfig.Host, config.ClickhouseConfig.Port)},
		Auth: clickhouse.Auth{
			Database: config.ClickhouseConfig.DB,
			Username: config.ClickhouseConfig.User,
			Password: config.ClickhouseConfig.Password,
		},
		TLS: &tls.Config{
			InsecureSkipVerify: true,
		},
	})

	if err != nil {
		logger.Error().Err(err).Msg("Failed to validate clickhouse connection")
		return err
	}

	logger.Debug().Msg("Trying to ping clickhouse")

	if err := conn.Ping(ctx); err != nil {
		logger.Error().Err(err).Msg("Failed to ping clickhouse")
		return err
	}
	logger.Info().Msg("Connected to clickhouse")
	c.Conn = conn
	return nil
}

var ClickhouseModule = fx.Module("clickhouse", fx.Provide(NewClickhouseDB), fx.Invoke(func(c *ClickhouseDB) {}))
