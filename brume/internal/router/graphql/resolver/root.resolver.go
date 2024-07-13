package resolver

import (
	"brume.dev/account/user"
	"brume.dev/project"
)

type RootResolver struct {
	userService    *user.UserService
	projectService *project.ProjectService
}

type QueryResolver struct {
	userService    *user.UserService
	projectService *project.ProjectService
}

type MutationResolver struct {
	userService    *user.UserService
	projectService *project.ProjectService
}

func NewRootResolver(userService *user.UserService, projectService *project.ProjectService) *RootResolver {
	return &RootResolver{
		userService:    userService,
		projectService: projectService,
	}
}

func (r *RootResolver) Query() *QueryResolver {
	return &QueryResolver{
		userService:    r.userService,
		projectService: r.projectService,
	}
}

func (r *RootResolver) Mutation() *MutationResolver {
	return &MutationResolver{
		userService:    r.userService,
		projectService: r.projectService,
	}
}
