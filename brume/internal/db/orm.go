package db

import (
	"time"

	org "github.com/brume/brume/org"
	"github.com/rs/zerolog/log"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

var AllModels = []interface{}{
	&org.Organization{},
}

type DB struct {
	gorm *gorm.DB
}

type Model struct {
	ID        uint `gorm:"primaryKey"`
	CreatedAt time.Time
	UpdatedAt time.Time
}

func InitDB() *DB {
	log.Info().Msg("Initializing database connection")
	db, err := openDB("brume.db")

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

	dialector := sqlite.Open(dsn)
	gorm, err := gorm.Open(dialector, &gorm.Config{
		Logger: dblogger,
	})

	if err != nil {
		return nil, err
	}

	db := &DB{
		gorm: gorm,
	}

	sqlDB, err := db.gorm.DB()
	if err != nil {
		return nil, err
	}

	sqlDB.SetMaxIdleConns(1)
	sqlDB.SetMaxOpenConns(1)

	return db, nil
}

func (db *DB) migrate() {
	// to add a model to migrate add it to the AllModels slice
	db.gorm.AutoMigrate(AllModels...)
}
