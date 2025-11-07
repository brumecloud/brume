package db

import (
	"fmt"

	"brume.dev/internal/config"
	brume_log "brume.dev/internal/log"
	"github.com/rs/zerolog"
	"go.uber.org/fx"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

var logger = brume_log.GetLogger("db")

var DBModule = fx.Module("db", fx.Provide(InitDB))

type DB struct {
	Gorm   *gorm.DB
	Config *config.BrumeConfig
}

func InitDB(config *config.BrumeConfig) *DB {
	dsn := fmt.Sprintf("user=%s password=%s dbname=%s host=%s sslmode=disable", config.PostgresConfig.User, config.PostgresConfig.Password, config.PostgresConfig.DB, config.PostgresConfig.Host)
	db, err := openDB(dsn, config)
	if err != nil {
		logger.Fatal().Err(err).Msg("Failed to open database connection")
	}

	logger.Info().Msg("Connected to the database")

	db.migrate(config)

	return db
}

func openDB(dsn string, config *config.BrumeConfig) (*DB, error) {
	logger.Info().Str("dsn", dsn).Msg("Opening database connection")

	level, err := zerolog.ParseLevel(config.PostgresConfig.Logs)
	if err != nil {
		level = zerolog.InfoLevel
	}

	dblogger := NewDBLogger(logger.Level(level))

	dialector := postgres.Open(dsn)
	gorm, err := gorm.Open(dialector, &gorm.Config{
		Logger: dblogger,
	})
	if err != nil {
		return nil, err
	}

	db := &DB{
		Gorm: gorm,
	}

	sqlDB, err := db.Gorm.DB()
	if err != nil {
		return nil, err
	}

	sqlDB.SetMaxIdleConns(config.PostgresConfig.MaxIdle)
	sqlDB.SetMaxOpenConns(config.PostgresConfig.MaxOpen)

	return db, nil
}
