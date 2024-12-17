package public_graph

import (
	"brume.dev/account/user"
	log "brume.dev/logs"
	"brume.dev/machine"
	"brume.dev/project"
	"brume.dev/service"
)

// This file will not be regenerated automatically.
//
// It serves as dependency injection for your app, add any dependencies you require here.

type Resolver struct {
	UserService    *user.UserService
	ProjectService *project.ProjectService
	ServiceService *service.ServiceService
	LogService     *log.LogService
	MachineService *machine.MachineService
}
