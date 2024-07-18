package resolver

import (
	"brume.dev/account/user"
	"brume.dev/project"
	"brume.dev/service"
)

type RootResolver struct {
	q              *QueryResolver
	m              *MutationResolver
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
	q              *QueryResolver
	userService    *user.UserService
	projectService *project.ProjectService
	serviceService *service.ServiceService
}

func NewRootResolver(userService *user.UserService, projectService *project.ProjectService, serviceService *service.ServiceService) *RootResolver {
	r := &RootResolver{
		userService:    userService,
		projectService: projectService,
		serviceService: serviceService,
	}
	r.q = r.NewQuery()
	r.m = r.NewMutation(r.q)
	return r
}

func (r *RootResolver) NewQuery() *QueryResolver {
	return &QueryResolver{
		userService:    r.userService,
		projectService: r.projectService,
		serviceService: r.serviceService,
	}
}

func (r *RootResolver) NewMutation(q *QueryResolver) *MutationResolver {
	return &MutationResolver{
		q:              q,
		userService:    r.userService,
		projectService: r.projectService,
		serviceService: r.serviceService,
	}
}

func (r *RootResolver) Query() *QueryResolver {
	return r.q
}

func (r *RootResolver) Mutation() *MutationResolver {
	return r.m
}
