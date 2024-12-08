package db

import (
	"errors"
	"time"

	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	builder_model "brume.dev/builder/model"
	project "brume.dev/project/model"
	runner_model "brume.dev/runner/model"
	service "brume.dev/service/model"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
	"golang.org/x/exp/rand"
	"gorm.io/gorm"
)

func SeedAll(db *DB) error {
	projects := SeedProjects(db)
	brume := SeedOrganization(db, projects)
	admin := SeedAdminUser(db, brume)

	_ = admin

	return nil
}

func SeedOrganization(db *DB, projects []*project.Project) *org.Organization {
	brume := &org.Organization{
		Name:     "brume",
		Projects: projects,
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

func generateDeployment(serviceId uuid.UUID) *service.Deployment {
	return &service.Deployment{
		ID:        uuid.New(),
		ServiceID: serviceId,
		Source: service.DeploymentSource{
			Type: service.DeploymentSourceTypeConsole,
		},
		DeployLog: service.DeploymentLog{
			Status:   service.DeploymentStatusSuccess,
			Duration: time.Duration(rand.Intn(100)) * time.Second,
			Date:     time.Now(),
		},
		RunnerData: runner_model.RunnerData{
			Command:        "",
			HealthCheckURL: "http://localhost:3000/health",
		},
		BuilderData: builder_model.BuilderData{
			Image:    "nginx",
			Registry: "docker.io",
			Tag:      "latest",
		},
		Env: "dev",
	}
}

func SeedProjects(db *DB) []*project.Project {
	projects := make([]*project.Project, 2)

	user_api_id := uuid.MustParse("2c77b616-fc35-4ab3-b4e9-0c57966dfd87")
	user_api := &service.Service{
		Name:        "User-API",
		ID:          user_api_id,
		Deployments: []*service.Deployment{},
		DraftBuilder: &builder_model.Builder{
			ID:   uuid.MustParse("f26a89ef-ff17-404a-96c5-3b03938c8149"),
			Type: "generic-docker",
			Data: builder_model.BuilderData{
				Image:    "nginx",
				Registry: "docker.io",
				Tag:      "latest",
			},
		},
		DraftRunner: &runner_model.Runner{
			ID:   uuid.MustParse("de56c895-c814-45fb-a859-ff943f293c3d"),
			Type: "generic-docker",
			Data: runner_model.RunnerData{
				Command:        "",
				HealthCheckURL: "http://localhost:3000/health",
				Memory: runner_model.RessourceConstraints{
					Request: 100,
					Limit:   100,
				},
				CPU: runner_model.RessourceConstraints{
					Request: 1,
					Limit:   1,
				},
				Port:          80,
				PublicDomain:  "user-api",
				PrivateDomain: "user-api",
			},
		},
	}

	stringID := "aaaaaaaa-91d1-4b9a-be84-b340e40614d3"
	id, _ := uuid.Parse(stringID)
	firstProject := &project.Project{
		Name:        "Porfolio",
		Description: "This is a test project",
		ID:          id,
		Services:    []*service.Service{user_api},
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

	open_ai_id := uuid.MustParse("a94cfd9e-5e61-4e5f-9fda-bb17d638a9ee")
	open_ai := &service.Service{
		Name:        "OpenAI-API",
		ID:          open_ai_id,
		Deployments: []*service.Deployment{},
		DraftBuilder: &builder_model.Builder{
			ID:   uuid.MustParse("4f6788dd-a317-4771-8afa-878b0b017b17"),
			Type: "generic-docker",
			Data: builder_model.BuilderData{
				Image:    "nginx",
				Registry: "docker.io",
				Tag:      "latest",
			},
		},
		DraftRunner: &runner_model.Runner{
			ID:   uuid.MustParse("6932f402-f633-48a9-bfc1-8489b1f3fd54"),
			Type: "generic-docker",
			Data: runner_model.RunnerData{
				Command:        "",
				HealthCheckURL: "http://localhost:3000/health",
				Memory: runner_model.RessourceConstraints{
					Request: 100,
					Limit:   100,
				},
				CPU: runner_model.RessourceConstraints{
					Request: 1,
					Limit:   1,
				},
				Port:          80,
				PublicDomain:  "openai-api",
				PrivateDomain: "openai-api",
			},
		},
	}

	wrapper_api_id := uuid.MustParse("b29dcba3-a2d3-40a5-bb70-2bd01002a062")
	wrapper_api := &service.Service{
		Name:        "Wrapper",
		ID:          wrapper_api_id,
		Deployments: []*service.Deployment{},
		DraftBuilder: &builder_model.Builder{
			Type: "generic-docker",
			ID:   uuid.MustParse("9376ac2a-ea1b-407b-a430-aabc0b687112"),
			Data: builder_model.BuilderData{
				Image:    "nginx",
				Registry: "docker.io",
				Tag:      "latest",
			},
		},
		DraftRunner: &runner_model.Runner{
			Name: "wrapper-runner",
			Type: "generic-docker",
			ID:   uuid.MustParse("e368e4c8-30b5-4eb2-9eb1-caf429984272"),
			Data: runner_model.RunnerData{
				Command:        "",
				HealthCheckURL: "http://localhost:3000/health",
				Memory: runner_model.RessourceConstraints{
					Request: 100,
					Limit:   100,
				},
				CPU: runner_model.RessourceConstraints{
					Request: 1,
					Limit:   1,
				},
				Port:          80,
				PublicDomain:  "wrapper",
				PrivateDomain: "wrapper",
			},
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
