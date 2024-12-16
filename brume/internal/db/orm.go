package db

import (
	"time"

	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	builder "brume.dev/builder/model"
	deployment_model "brume.dev/deployment/model"
	project_model "brume.dev/project/model"
	runner "brume.dev/runner/model"
	service_model "brume.dev/service/model"
	"github.com/rs/zerolog/log"
)

var AllModels = []interface{}{
	&org.Organization{},
	&user.User{},
	&service_model.Service{},
	&deployment_model.Deployment{},
	&project_model.Project{},
	&runner.Runner{},
	&builder.Builder{},
}

type Model struct {
	ID        uint `gorm:"primaryKey"`
	CreatedAt time.Time
	UpdatedAt time.Time
}

func (db *DB) migrate() {
	log.Info().Msg("Starting the migration")
	// to add a model to migrate add it to the AllModels slice
	db.Gorm.AutoMigrate(AllModels...)
	log.Info().Msg("All migrations passed, continuing with seeding")

	SeedAll(db)
	log.Info().Msg("Seeding finished")
}
