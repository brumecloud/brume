package db

import (
	"time"

	log_model "brume.dev/logs/model"
	"github.com/rs/zerolog/log"
	"gorm.io/driver/clickhouse"
	"gorm.io/gorm"
)

type ClickhouseDB struct {
	Gorm *gorm.DB
}

func InitClickhouse() *ClickhouseDB {
	log.Info().Msg("Initializing Clickhouse connection")

	chdb, err := openCHDB()

	if err != nil {
		log.Fatal().Err(err).Msg("Failed to connect to Clickhouse")
	}

	chdb.Migrate()

	return chdb
}

func openCHDB() (*ClickhouseDB, error) {
	log.Info().Msg("Opening the clickhouse database connection")
	globalLogLevel := log.Logger.GetLevel()
	dblogger := NewDBLogger(log.Level(globalLogLevel))

	dsn := "clickhouse://brume:brumepass@clickhouse:9000/brume?dial_timeout=10s&read_timeout=20s"

	db, err := gorm.Open(clickhouse.Open(dsn), &gorm.Config{
		Logger: dblogger,
	})

	if err != nil {
		log.Fatal().Err(err).Msg("Failed to connect to Clickhouse")
	}

	db.Set("gorm:table_options", "ENGINE=Distributed(cluster, default, hits)")

	sqlDB, err := db.DB()

	if err != nil {
		log.Fatal().Err(err).Msg("Failed to get sqlDB from Clickhouse")
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
	log.Info().Msg("Migrating Clickhouse database")

	chdb.Gorm.AutoMigrate(AllClickhouseModels...)

	log.Info().Msg("Clickhouse migrations finished")
}
