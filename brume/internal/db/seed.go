package db

import (
	"errors"
	"os"

	agent_model "brume.dev/account/agent/model"
	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	builder_model "brume.dev/builder/model"
	cloud_account_model "brume.dev/cloud/account/model"
	"brume.dev/internal/config"
	project "brume.dev/project/model"
	runner_model "brume.dev/runner/model"
	service "brume.dev/service/model"
	source_model "brume.dev/source/model"
	stack_model "brume.dev/stack/model"
	"gorm.io/gorm"
)

func SeedAll(db *DB, config *config.BrumeConfig) error {
	projects := SeedProjects(db)
	brume := SeedOrganization(db, projects, config)
	cloudAccounts := SeedCloudAccounts(db, brume)
	admin := SeedAdminUser(db, brume, config)
	_, _ = SeedAgent(db, brume, config)
	SeedStackTemplates(db)

	_ = admin
	_ = cloudAccounts

	return nil
}

func SeedOrganization(db *DB, projects []*project.Project, config *config.BrumeConfig) *org.Organization {
	brume := &org.Organization{
		ID:       config.BrumeGeneralConfig.StaffOrgID,
		Name:     "BrumeCloud",
		Projects: projects,
	}

	if err := db.Gorm.First(brume, "name = ?", "BrumeCloud").Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Debug().Msg("No organization found in database, creating brume")
		db.Gorm.Create(brume)
		logger.Debug().Msg("Organization seeded")
	} else {
		logger.Debug().Msg("Organization found, skipping seeding")
	}

	return brume
}

func SeedAdminUser(db *DB, brume *org.Organization, config *config.BrumeConfig) *user.User {
	admin := &user.User{
		ProviderID:     config.BrumeGeneralConfig.SudoProviderID,
		Name:           "Paul Planchon",
		OrganizationID: brume.ID,
		Avatar:         "https://avatars.githubusercontent.com/u/34143515?v=4",
	}

	if err := db.Gorm.First(admin, "provider_id = ?", config.BrumeGeneralConfig.SudoProviderID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Debug().Msg("No user found in database, creating sudo user")

		db.Gorm.Create(admin)
		logger.Debug().Msg("Admin user seeded")
	} else {
		logger.Debug().Msg("Admin user found, skipping seeding")
	}

	return admin
}

func SeedAgent(db *DB, brume *org.Organization, config *config.BrumeConfig) (*agent_model.Agent, *agent_model.Agent) {
	runnerAgent := &agent_model.Agent{
		ID:             "agnt_339fa4e0bd17",
		APIKey:         "runner-api-key",
		AgentType:      agent_model.AgentTypeRunner,
		OrganizationID: brume.ID,
	}
	builderAgent := &agent_model.Agent{
		ID:             "agnt_1590234b2dbb",
		APIKey:         "builder-api-key",
		AgentType:      agent_model.AgentTypeBuilder,
		OrganizationID: brume.ID,
	}

	if err := db.Gorm.First(runnerAgent, "id = ?", runnerAgent.ID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Debug().Msg("No runner agent found in database, creating it")
		db.Gorm.Create(runnerAgent)
		logger.Debug().Msg("Runner agent seeded")
	} else {
		logger.Debug().Msg("Runner agent found, skipping seeding")
	}

	if err := db.Gorm.First(builderAgent, "id = ?", builderAgent.ID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Debug().Msg("No builder agent found in database, creating it")
		db.Gorm.Create(builderAgent)
		logger.Debug().Msg("Builder agent seeded")
	} else {
		logger.Debug().Msg("Builder agent found, skipping seeding")
	}

	return runnerAgent, builderAgent
}

func SeedProjects(db *DB) []*project.Project {
	projects := make([]*project.Project, 1)

	frontendId := "serv_8e395e26117e"

	// read the builder.json file
	builderJsonRaw, _ := os.ReadFile("internal/db/jsons/builder.json")
	runnerJsonRaw, _ := os.ReadFile("internal/db/jsons/runner.json")
	builderJson := string(builderJsonRaw)
	runnerJson := string(runnerJsonRaw)

	frontend := &service.Service{
		Name: "Frontend",
		ID:   frontendId,
		Live: &service.BaseService{
			Source: &source_model.Source{
				ID:   frontendId,
				Type: source_model.SourceTypeGit,
				GitData: &source_model.GitSource{
					Provider:   "github",
					Repository: "brume-dev/brume-portfolio",
					Branch:     "main",
				},
			},
			Runner: &runner_model.Runner{
				ID:     frontendId,
				Type:   "spa-cloudfront",
				Schema: runnerJson,
				Data: `{
					"automatic": {
						"bucket_path": "brume-portfolio",
						"folder_path": "frontend",
					},
					"manual": {},
				}`,
			},
			Builder: &builder_model.Builder{
				ID:     frontendId,
				Type:   "spa",
				Schema: builderJson,
				Data: `{
					"automatic": {
						"bucket_path": "brume-portfolio",
						"folder_path": "frontend",
					},
					"manual": {
						"install_command": "npm install",
						"build_command": "npm run build",
						"output_path": "dist",
					},
				}`,
			},
		},
	}

	project := &project.Project{
		Name:        "Brume Test Project",
		Description: "This is a test project",
		ID:          "proj_3f29ea37c238",
		Services:    []*service.Service{frontend},
	}

	if err := db.Gorm.First(project, "id = ?", project.ID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Debug().Msg("Porfolio project not found, creating it")

		db.Gorm.Create(project)
		logger.Debug().Msg("Porfolio project created")
	} else {
		logger.Debug().Msg("Porfolio project found, skipping seeding")
	}

	projects[0] = project

	return projects
}

func SeedStackTemplates(db *DB) {
	stackTemplate := &stack_model.StackTemplate{
		ID:   "stk_tpl_d32a242475c5",
		Name: "Dummy Cloud front SPA",
	}

	if err := db.Gorm.First(stackTemplate, "id = ?", stackTemplate.ID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Debug().Msg("No stack template found in database, creating it")
		db.Gorm.Create(stackTemplate)
		logger.Debug().Msg("Stack template seeded")
	} else {
		logger.Debug().Msg("Stack template found, skipping seeding")
	}
}

func SeedCloudAccounts(db *DB, brume *org.Organization) []*cloud_account_model.CloudAccount {
	cloudAccounts := make([]*cloud_account_model.CloudAccount, 1)

	stacks := make([]*stack_model.Stack, 1)

	stack := &stack_model.Stack{
		ID:   "stck_3f29ea37c238",
		Name: "Dummy Cloud front SPA",
	}

	stacks[0] = stack

	cloudAccount := &cloud_account_model.CloudAccount{
		ID:            "ca_d5736d9338a5",
		Status:        cloud_account_model.CloudStatusConnected,
		Name:          "Dummy AWS Dev account",
		Description:   "This is the development AWS account",
		CloudProvider: cloud_account_model.CloudProviderAWS,
		AWS: &cloud_account_model.AWSCloudAccount{
			AccountID: "123456789012",
		},
		Stacks:         stacks,
		OrganizationID: brume.ID,
	}

	if err := db.Gorm.First(cloudAccount, "id = ?", cloudAccount.ID).Error; errors.Is(err, gorm.ErrRecordNotFound) {
		logger.Debug().Msg("No cloud account found in database, creating it")
		db.Gorm.Create(cloudAccount)
		logger.Debug().Msg("Cloud account seeded")
	} else {
		logger.Debug().Msg("Cloud account found, skipping seeding")
	}

	cloudAccounts[0] = cloudAccount
	return cloudAccounts
}
