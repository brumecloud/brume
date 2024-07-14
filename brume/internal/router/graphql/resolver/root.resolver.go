package resolver

import (
	"brume.dev/account/user"
	"brume.dev/project"
	"brume.dev/service"
)

type RootResolver struct {
	userService    *user.UserService
	projectService *project.ProjectService
	serviceService *service.ServiceService
}

type QueryResolver struct {
	userService    *user.UserService
	projectService *project.ProjectService
	serviceService *service.ServiceService
}

type MutationResolver struct {
	userService    *user.UserService
	projectService *project.ProjectService
	serviceService *service.ServiceService
}

func NewRootResolver(userService *user.UserService, projectService *project.ProjectService, serviceService *service.ServiceService) *RootResolver {
	return &RootResolver{
		userService:    userService,
		projectService: projectService,
		serviceService: serviceService,
	}
}

func (r *RootResolver) Query() *QueryResolver {
	return &QueryResolver{
		userService:    r.userService,
		projectService: r.projectService,
		serviceService: r.serviceService,
	}
}

func (r *RootResolver) Mutation() *MutationResolver {
	return &MutationResolver{
		userService:    r.userService,
		projectService: r.projectService,
		serviceService: r.serviceService,
	}
}
