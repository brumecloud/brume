package db

import (
	"errors"

	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	builder_model "brume.dev/builder/model"
	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/config"
	machine_model "brume.dev/machine/model"
	project "brume.dev/project/model"
	runner_model "brume.dev/runner/model"
	service "brume.dev/service/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

func SeedAll(db *DB, config *config.BrumeConfig) error {
	projects := SeedProjects(db)
	brume := SeedOrganization(db, projects, config)
	admin := SeedAdminUser(db, brume, config)
	machine := SeedMachine(db, brume)

	_ = admin
	_ = machine

	return nil
}

func SeedOrganization(db *DB, projects []*project.Project, config *config.BrumeConfig) *org.Organization {
	brume := &org.Organization{
		Name:       "BrumeCloud",
		ProviderID: config.BrumeGeneralConfig.StaffOrgID,
		Projects:   projects,
	}

	if err := db.Gorm.First(brume, "name = ?", "brume").Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Info().Msg("No organization found in database, creating brume")
		db.Gorm.Create(brume)
		logger.Info().Msg("Organization seeded")
	} else {
		logger.Info().Msg("Organization found, skipping seeding")
	}

	return brume
}

func SeedMachine(db *DB, brume *org.Organization) *machine_model.Machine {
	machine := &machine_model.Machine{
		ID:             uuid.MustParse("b36d84e9-bec2-4ba1-8b51-536884f06bc7"),
		Name:           "docker-local-machine",
		IP:             "127.0.0.1",
		OrganizationID: brume.ID,
	}

	if err := db.Gorm.First(machine, "id = ?", machine.ID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Info().Msg("No machine found in database, creating brume-machine")
		err := db.Gorm.Create(machine).Error
		if err != nil {
			logger.Error().Err(err).Msg("Error seeding machine")
		}

		logger.Info().Msg("Machine seeded")
	} else {
		logger.Info().Msg("Machine found, skipping seeding")
	}

	return machine
}

func SeedAdminUser(db *DB, brume *org.Organization, config *config.BrumeConfig) *user.User {
	admin := &user.User{
		ProviderID:     config.BrumeGeneralConfig.SudoProviderID,
		Name:           "Paul Planchon",
		OrganizationID: brume.ID,
		Avatar:         "https://avatars.githubusercontent.com/u/34143515?v=4",
	}

	if err := db.Gorm.First(admin, "provider_id = ?", config.BrumeGeneralConfig.SudoProviderID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Info().Msg("No user found in database, creating sudo user")

		db.Gorm.Create(admin)
		logger.Info().Msg("Admin user seeded")
	} else {
		logger.Info().Msg("Admin user found, skipping seeding")
	}

	return admin
}

func generateDeployment(serviceId uuid.UUID) *deployment_model.Deployment {
	return &deployment_model.Deployment{
		ID:        uuid.New(),
		ServiceID: serviceId,
		Source: deployment_model.DeploymentSource{
			Type: deployment_model.DeploymentSourceTypeConsole,
		},
		RunnerData: runner_model.RunnerData{
			Type: runner_model.RunnerTypeDocker,
			Docker: &runner_model.DockerRunnerData{
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
				Port: 80,
			},
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
	projects := make([]*project.Project, 1)

	user_api_id := uuid.MustParse("2c77b616-fc35-4ab3-b4e9-0c57966dfd87")
	user_api := &service.Service{
		Name:        "User-API",
		ID:          user_api_id,
		Deployments: []*deployment_model.Deployment{},
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
				Type: runner_model.RunnerTypeDocker,
				Docker: &runner_model.DockerRunnerData{
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
					Port: 80,
				},
				PublicDomain:  "user-api",
				PrivateDomain: nil,
			},
		},
	}

	frontend := &service.Service{
		Name:        "Frontend",
		ID:          uuid.MustParse("536b85d4-53ff-4a8f-b3f3-ec134257adb9"),
		Deployments: []*deployment_model.Deployment{},
		DraftBuilder: &builder_model.Builder{
			ID:   uuid.MustParse("21e0d32e-a6a4-471b-b7e3-2201770dfeb8"),
			Type: "generic-docker",
			Data: builder_model.BuilderData{
				Image:    "nginx",
				Registry: "docker.io",
				Tag:      "latest",
			},
		},
		DraftRunner: &runner_model.Runner{
			ID:   uuid.MustParse("c73eb9cf-9143-401a-9c56-b035f1561470"),
			Type: "generic-docker",
			Data: runner_model.RunnerData{
				Type: runner_model.RunnerTypeDocker,
				Docker: &runner_model.DockerRunnerData{
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
					Port: 80,
				},
				PublicDomain:  "frontend",
				PrivateDomain: nil,
			},
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
		logger.Info().Msg("Porfolio project not found, creating it")

		firstProjectError := db.Gorm.Create(firstProject).Error
		if firstProjectError != nil {
			logger.Error().Msg(firstProjectError.Error())
		}
		logger.Info().Msg("Porfolio project created")
	}

	projects[0] = firstProject

	// open_ai_id := uuid.MustParse("a94cfd9e-5e61-4e5f-9fda-bb17d638a9ee")
	// open_ai := &service.Service{
	// 	Name:        "OpenAI-API",
	// 	ID:          open_ai_id,
	// 	Deployments: []*deployment_model.Deployment{},
	// 	DraftBuilder: &builder_model.Builder{
	// 		ID:   uuid.MustParse("4f6788dd-a317-4771-8afa-878b0b017b17"),
	// 		Type: "generic-docker",
	// 		Data: builder_model.BuilderData{
	// 			Image:    "nginx",
	// 			Registry: "docker.io",
	// 			Tag:      "latest",
	// 		},
	// 	},
	// 	DraftRunner: &runner_model.Runner{
	// 		ID:   uuid.MustParse("6932f402-f633-48a9-bfc1-8489b1f3fd54"),
	// 		Type: "generic-docker",
	// 		Data: runner_model.RunnerData{
	// 			Type: runner_model.RunnerTypeDocker,
	// 			Docker: runner_model.DockerRunnerData{
	// 				Command:        "",
	// 				HealthCheckURL: "http://localhost:3000/health",
	// 				Memory: runner_model.RessourceConstraints{
	// 					Request: 100,
	// 					Limit:   100,
	// 				},
	// 				CPU: runner_model.RessourceConstraints{
	// 					Request: 1,
	// 					Limit:   1,
	// 				},
	// 				Port: 80,
	// 			},
	// 			PublicDomain:  "openai-api",
	// 			PrivateDomain: "openai-api",
	// 		},
	// 	},
	// }

	// wrapper_api_id := uuid.MustParse("b29dcba3-a2d3-40a5-bb70-2bd01002a062")
	// wrapper_api := &service.Service{
	// 	Name:        "Wrapper",
	// 	ID:          wrapper_api_id,
	// 	Deployments: []*deployment_model.Deployment{},
	// 	DraftBuilder: &builder_model.Builder{
	// 		Type: "generic-docker",
	// 		ID:   uuid.MustParse("9376ac2a-ea1b-407b-a430-aabc0b687112"),
	// 		Data: builder_model.BuilderData{
	// 			Image:    "nginx",
	// 			Registry: "docker.io",
	// 			Tag:      "latest",
	// 		},
	// 	},
	// 	DraftRunner: &runner_model.Runner{
	// 		Name: "wrapper-runner",
	// 		Type: "generic-docker",
	// 		ID:   uuid.MustParse("e368e4c8-30b5-4eb2-9eb1-caf429984272"),
	// 		Data: runner_model.RunnerData{
	// 			Type: runner_model.RunnerTypeDocker,
	// 			Docker: runner_model.DockerRunnerData{
	// 				Command:        "",
	// 				HealthCheckURL: "http://localhost:3000/health",
	// 				Memory: runner_model.RessourceConstraints{
	// 					Request: 100,
	// 					Limit:   100,
	// 				},
	// 				CPU: runner_model.RessourceConstraints{
	// 					Request: 1,
	// 					Limit:   1,
	// 				},
	// 				Port: 80,
	// 			},
	// 			PublicDomain:  "wrapper",
	// 			PrivateDomain: "wrapper",
	// 		},
	// 	},
	// }

	// stringID = "bbbbbbbb-91d1-4b9a-be84-b340e40614d3"
	// id, _ = uuid.Parse(stringID)
	// secondProject := &project.Project{
	// 	ID:          id,
	// 	Name:        "GenAI",
	// 	Description: "This is a test project",
	// 	Services:    []*service.Service{open_ai, wrapper_api},
	// }

	// if err := db.Gorm.First(secondProject, "id = ?", stringID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
	// 	logger.Info().Msg("GenAI project not found, creating it")

	// 	db.Gorm.Create(secondProject)
	// 	logger.Info().Msg("GenAI project created")
	// }

	// projects[1] = secondProject

	return projects
}
