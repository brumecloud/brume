package db

import (
	brumelog "github.com/brumecloud/agent/internal/log"
	running_job "github.com/brumecloud/agent/job/model"
	_ "github.com/ncruces/go-sqlite3/embed"
	"github.com/ncruces/go-sqlite3/gormlite"

	"gorm.io/gorm"
)

var logger = brumelog.GetLogger().With().Str("module", "db").Logger()

type DB struct {
	Gorm *gorm.DB
}

func InitDB() *DB {
	db, err := openDB("user=brume password=brumepass dbname=brume host=postgres sslmode=disable")
	if err != nil {
		logger.Fatal().Err(err).Msg("Failed to open database connection")
	}

	logger.Info().Msg("Migrating SQLite database")
	db.Gorm.AutoMigrate(&running_job.RunningJob{})
	logger.Info().Msg("Migration complete")

	return db
}

func openDB(dsn string) (*DB, error) {
	logger.Info().Str("dsn", dsn).Msg("Opening database connection")
	globalLogLevel := logger.GetLevel()
	dblogger := NewDBLogger(brumelog.GetLogger().Level(globalLogLevel))

	// sqlite db
	db, err := gorm.Open(gormlite.Open("agent.db"), &gorm.Config{
		Logger: dblogger,
	})
	if err != nil {
		return nil, err
	}

	return &DB{Gorm: db}, nil
}
