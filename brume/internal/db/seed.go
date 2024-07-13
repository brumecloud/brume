package db

import (
	"errors"

	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	project "brume.dev/project/model"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
	"gorm.io/gorm"
)

func SeedAll(db *DB) error {
	brume := SeedOrganization(db)
	admin := SeedAdminUser(db, brume)
	SeedProjects(db)

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

func SeedAdminUser(db *DB, brume *org.Organization) *user.User {
	admin := &user.User{
		Email:          "admin@brume.dev",
		Name:           "Brume Admin",
		Password:       "adminpass",
		OrganizationID: brume.ID,
		Avatar:         "https://avatars.githubusercontent.com/u/34143515?v=4",
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

func SeedProjects(db *DB) {
	stringID := "aaaaaaaa-91d1-4b9a-be84-b340e40614d3"
	id, _ := uuid.Parse(stringID)
	firstProject := &project.Project{
		Name:        "Porfolio",
		Description: "This is a test project",
		ID:          id,
	}

	if err := db.Gorm.First(firstProject, "id = ?", stringID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		log.Info().Msg("Porfolio project not found, creating it")

		db.Gorm.Create(firstProject)
		log.Info().Msg("Porfolio project created")
	}

	stringID = "bbbbbbbb-91d1-4b9a-be84-b340e40614d3"
	id, _ = uuid.Parse(stringID)
	secondProject := &project.Project{
		ID:          id,
		Name:        "GenAI",
		Description: "This is a test project",
	}

	if err := db.Gorm.First(secondProject, "id = ?", stringID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		log.Info().Msg("GenAI project not found, creating it")

		db.Gorm.Create(secondProject)
		log.Info().Msg("GenAI project created")
	}
}
