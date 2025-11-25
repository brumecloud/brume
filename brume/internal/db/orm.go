package db

import (
	"time"

	agent_model "brume.dev/account/agent/model"
	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	cloud_account_model "brume.dev/cloud/account/model"
	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/config"
	job_model "brume.dev/jobs/model"
	project_model "brume.dev/project/model"
	service_model "brume.dev/service/model"
	stack_model "brume.dev/stack/model"
)

var AllModels = []interface{}{
	&org.Organization{},
	&user.User{},
	&service_model.Service{},
	&deployment_model.Deployment{},
	&project_model.Project{},
	&job_model.Job{},
	&agent_model.Agent{},
	&cloud_account_model.CloudAccount{},
	&stack_model.Stack{},
	&stack_model.StackTemplate{},
}

type Model struct {
	ID        uint `gorm:"primaryKey"`
	CreatedAt time.Time
	UpdatedAt time.Time
}

func (db *DB) migrate(config *config.BrumeConfig) {
	logger.Info().Msg("Starting the migration")

	// to add a model to migrate add it to the AllModels slice
	db.Gorm.AutoMigrate(AllModels...)
	logger.Info().Msg("All migrations passed, continuing with seeding")

	SeedAll(db, config)
	logger.Info().Msg("Seeding finished")
}
