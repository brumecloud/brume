package public_graph

import (
	"brume.dev/account/org"
	"brume.dev/account/user"
	cloud_account_service "brume.dev/cloud/account"
	"brume.dev/internal/config"
	log "brume.dev/logs"
	"brume.dev/machine"
	"brume.dev/project"
	"brume.dev/service"
	stack_service "brume.dev/stack"
)

// This file will not be regenerated automatically.
//
// It serves as dependency injection for your app, add any dependencies you require here.

type Resolver struct {
	UserService         *user.UserService
	ProjectService      *project.ProjectService
	ServiceService      *service.ServiceService
	LogService          *log.LogService
	MachineService      *machine.MachineService
	ConfigService       *config.BrumeConfig
	CloudAccountService *cloud_account_service.CloudAccountService
	OrganizationService *org.OrganizationService
	StackService        *stack_service.StackService
}
