package db

import (
	"errors"
	"time"

	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	project "brume.dev/project/model"
	service "brume.dev/service/model"
	"github.com/rs/zerolog/log"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

var AllModels = []interface{}{
	&org.Organization{},
	&user.User{},
	&service.Service{},
	&project.Project{},
}

type DB struct {
	Gorm *gorm.DB
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

func (db *DB) migrate() {
	// to add a model to migrate add it to the AllModels slice
	db.Gorm.AutoMigrate(AllModels...)

	brume := &org.Organization{
		Name: "brume",
	}

	if err := db.Gorm.First(&org.Organization{}, "name = ?", "brume").Error; errors.Is(err, gorm.ErrRecordNotFound) {
		log.Info().Msg("No organization found in database, creating brume")
		db.Gorm.Create(brume)
		log.Info().Msg("Organization seeded")
	} else {
		log.Info().Msg("Organization found, skipping seeding")
	}

	admin := &user.User{
		Email:          "admin@brume.dev",
		Password:       "adminpass",
		OrganizationID: brume.ID,
	}

	if err := db.Gorm.First(&user.User{}, "email = ?", "admin@brume.dev").Error; errors.Is(err, gorm.ErrRecordNotFound) {
		log.Info().Msg("No user found in database, creating admin@brume.dev")

		db.Gorm.Create(admin)
		log.Info().Msg("Admin user seeded")
	} else {
		log.Info().Msg("Admin user found, skipping seeding")
	}
}
