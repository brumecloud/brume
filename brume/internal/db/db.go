package db

import (
	"github.com/rs/zerolog/log"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

type DB struct {
	Gorm *gorm.DB
}

func InitDB() *DB {
	log.Info().Msg("Initializing database connection")
	db, err := openDB("user=brume password=brumepass dbname=brume host=postgres sslmode=disable")

	if err != nil {
		log.Fatal().Err(err).Msg("Failed to open database connection")
	}

	db.migrate()

	return db
}

func openDB(dsn string) (*DB, error) {
	log.Info().Str("dsn", dsn).Msg("Opening database connection")
	globalLogLevel := log.Logger.GetLevel()
	dblogger := NewDBLogger(log.Level(globalLogLevel))

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

	sqlDB.SetMaxIdleConns(1)
	sqlDB.SetMaxOpenConns(1)

	return db, nil
}
