package db

import (
	"errors"

	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	project "brume.dev/project/model"
	runner_model "brume.dev/runner/model"
	service "brume.dev/service/model"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
	"gorm.io/gorm"
)

func SeedAll(db *DB) error {
	brume := SeedOrganization(db)
	projects := SeedProjects(db)
	admin := SeedAdminUser(db, brume, projects)

	_ = admin

	return nil
}

func SeedOrganization(db *DB) *org.Organization {
	brume := &org.Organization{
		Name: "brume",
	}

	if err := db.Gorm.First(brume, "name = ?", "brume").Error; errors.Is(err, gorm.ErrRecordNotFound) {
		log.Info().Msg("No organization found in database, creating brume")
		db.Gorm.Create(brume)
		log.Info().Msg("Organization seeded")
	} else {
		log.Info().Msg("Organization found, skipping seeding")
	}

	return brume
}

func SeedAdminUser(db *DB, brume *org.Organization, projects []*project.Project) *user.User {
	admin := &user.User{
		Email:          "admin@brume.dev",
		Name:           "Brume Admin",
		Password:       "adminpass",
		OrganizationID: brume.ID,
		Avatar:         "https://avatars.githubusercontent.com/u/34143515?v=4",
		Projects:       projects,
	}

	if err := db.Gorm.First(admin, "email = ?", "admin@brume.dev").Error; errors.Is(err, gorm.ErrRecordNotFound) {
		log.Info().Msg("No user found in database, creating admin@brume.dev")

		db.Gorm.Create(admin)
		log.Info().Msg("Admin user seeded")
	} else {
		log.Info().Msg("Admin user found, skipping seeding")
	}

	return admin
}

func SeedProjects(db *DB) []*project.Project {
	projects := make([]*project.Project, 2)

	user_api := &service.Service{
		Name: "User-API",
		ID:   uuid.MustParse("2c77b616-fc35-4ab3-b4e9-0c57966dfd87"),
		Runner: &runner_model.Runner{
			ID:    uuid.MustParse("aeb51cc0-e6a7-4eb3-8199-3d6a94070548"),
			Name:  "user-api-runner",
			Image: "hello-world",
		},
	}
	frontend := &service.Service{
		Name: "Frontend",
		ID:   uuid.MustParse("1c45217f-2f15-496d-a5cf-7860fec720e3"),
		Runner: &runner_model.Runner{
			ID:    uuid.MustParse("84127be1-524f-46cf-8f18-fc9e725a3a0f"),
			Name:  "frontend-runner",
			Image: "hello-world",
		},
	}

	stringID := "aaaaaaaa-91d1-4b9a-be84-b340e40614d3"
	id, _ := uuid.Parse(stringID)
	firstProject := &project.Project{
		Name:        "Porfolio",
		Description: "This is a test project",
		ID:          id,
		Services:    []*service.Service{user_api, frontend},
	}

	if err := db.Gorm.First(firstProject, "id = ?", stringID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		log.Info().Msg("Porfolio project not found, creating it")

		firstProjectError := db.Gorm.Create(firstProject).Error
		if firstProjectError != nil {
			log.Error().Msg(firstProjectError.Error())
		}
		log.Info().Msg("Porfolio project created")
	}

	projects[0] = firstProject

	open_ai := &service.Service{
		Name: "OpenAI-API",
		ID:   uuid.MustParse("a94cfd9e-5e61-4e5f-9fda-bb17d638a9ee"),
		Runner: &runner_model.Runner{
			ID:    uuid.MustParse("b2e8637c-ebe2-49d2-92cc-9b103a5bbcbc"),
			Name:  "wrapper-runner",
			Image: "hello-world",
		},
	}
	wrapper_api := &service.Service{
		Name: "Wrapper",
		ID:   uuid.MustParse("b29dcba3-a2d3-40a5-bb70-2bd01002a062"),
		Runner: &runner_model.Runner{
			ID:    uuid.MustParse("d1f3c453-28d6-4b20-85d9-5e1040e4a448"),
			Name:  "wrapper-runner",
			Image: "hello-world",
		},
	}

	stringID = "bbbbbbbb-91d1-4b9a-be84-b340e40614d3"
	id, _ = uuid.Parse(stringID)
	secondProject := &project.Project{
		ID:          id,
		Name:        "GenAI",
		Description: "This is a test project",
		Services:    []*service.Service{open_ai, wrapper_api},
	}

	if err := db.Gorm.First(secondProject, "id = ?", stringID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		log.Info().Msg("GenAI project not found, creating it")

		db.Gorm.Create(secondProject)
		log.Info().Msg("GenAI project created")
	}

	projects[1] = secondProject

	return projects
}
