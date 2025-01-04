package brume_clickhouse

import (
	"fmt"
	"time"

	config "brume.dev/internal/config"
	db "brume.dev/internal/db"
	brume_log "brume.dev/internal/log"
	log_model "brume.dev/logs/model"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
	"gorm.io/driver/clickhouse"
	"gorm.io/gorm"
)

var logger = brume_log.GetLogger("clickhouse")

type ClickhouseDB struct {
	Gorm *gorm.DB
}

func InitClickhouse(cfg *config.BrumeConfig) *ClickhouseDB {
	logger.Info().Msg("Initializing Clickhouse connection")

	chdb, err := openCHDB(cfg)
	if err != nil {
		logger.Fatal().Err(err).Msg("Failed to connect to Clickhouse")
	}

	chdb.Migrate()

	return chdb
}

func openCHDB(cfg *config.BrumeConfig) (*ClickhouseDB, error) {
	globalLogLevel := logger.GetLevel()
	dblogger := db.NewDBLogger(log.Level(globalLogLevel))

	dsn := fmt.Sprintf("clickhouse://%s:%s@%s:%d/%s?dial_timeout=10s&read_timeout=20s", cfg.ClickhouseUser, cfg.ClickhousePassword, cfg.ClickhouseHost, cfg.ClickhousePort, cfg.ClickhouseDB)

	logger.Info().Str("dsn", dsn).Msg("Opening the clickhouse database connection")

	db, err := gorm.Open(clickhouse.Open(dsn), &gorm.Config{
		Logger: dblogger,
	})
	if err != nil {
		logger.Fatal().Err(err).Msg("Failed to connect to Clickhouse")
	}

	db.Set("gorm:table_options", "ENGINE=Distributed(cluster, default, hits)")

	sqlDB, err := db.DB()
	if err != nil {
		logger.Fatal().Err(err).Msg("Failed to get sqlDB from Clickhouse")
	}

	sqlDB.SetMaxIdleConns(10)
	sqlDB.SetMaxOpenConns(100)
	sqlDB.SetConnMaxLifetime(time.Hour)

	return &ClickhouseDB{
		Gorm: db,
	}, nil
}

var AllClickhouseModels = []interface{}{
	&log_model.Log{},
}

func (chdb *ClickhouseDB) Migrate() {
	logger.Info().Msg("Migrating Clickhouse database")
	chdb.Gorm.AutoMigrate(AllClickhouseModels...)
	logger.Info().Msg("Clickhouse migrations finished")
}

var ClickhouseModule = fx.Module("clickhouse", fx.Provide(InitClickhouse))
