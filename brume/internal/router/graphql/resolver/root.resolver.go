package resolver

import (
	"brume.dev/account/user"
	log "brume.dev/logs"
	"brume.dev/project"
	"brume.dev/service"
)

type RootResolver struct {
	q              *QueryResolver
	m              *MutationResolver
	s              *SubscriptionResolver
	userService    *user.UserService
	projectService *project.ProjectService
	serviceService *service.ServiceService
	logService     *log.LogService
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

type SubscriptionResolver struct {
	logService *log.LogService
}

func NewRootResolver(userService *user.UserService, projectService *project.ProjectService, serviceService *service.ServiceService, logService *log.LogService) *RootResolver {
	r := &RootResolver{
		userService:    userService,
		projectService: projectService,
		serviceService: serviceService,
		logService:     logService,
	}
	r.q = r.NewQuery()
	r.m = r.NewMutation(r.q)
	r.s = r.NewSubscription()
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

func (r *RootResolver) NewSubscription() *SubscriptionResolver {
	return &SubscriptionResolver{
		logService: r.logService,
	}
}

func (r *RootResolver) Query() *QueryResolver {
	return r.q
}

func (r *RootResolver) Mutation() *MutationResolver {
	return r.m
}

func (r *RootResolver) Subscription() *SubscriptionResolver {
	return r.s
}
