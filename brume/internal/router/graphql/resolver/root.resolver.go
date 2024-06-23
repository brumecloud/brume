package resolver

import "brume.dev/account/user"

type RootResolver struct {
	userService *user.UserService
}

type QueryResolver struct {
	userService *user.UserService
}

type MutationResolver struct {
	userService *user.UserService
}

func NewRootResolver(userService *user.UserService) *RootResolver {
	return &RootResolver{
		userService: userService,
	}
}

func (r *RootResolver) Query() *QueryResolver {
	return &QueryResolver{
		userService: r.userService,
	}
}

func (r *RootResolver) Mutation() *MutationResolver {
	return &MutationResolver{
		userService: r.userService,
	}
}
