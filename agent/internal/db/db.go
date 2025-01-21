package db

import (
	brumelog "github.com/brumecloud/agent/internal/log"
	"gorm.io/driver/sqlite"
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

	return db
}

func openDB(dsn string) (*DB, error) {
	logger.Info().Str("dsn", dsn).Msg("Opening database connection")
	globalLogLevel := logger.GetLevel()
	dblogger := NewDBLogger(brumelog.GetLogger().Level(globalLogLevel))

	// sqlite db
	db, err := gorm.Open(sqlite.Open("file::memory:?cache=shared"), &gorm.Config{
		Logger: dblogger,
	})
	if err != nil {
		return nil, err
	}

	return &DB{Gorm: db}, nil
}
