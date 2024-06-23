package resolver

type RootResolver struct{}
type QueryResolver struct{}
type MutationResolver struct{}

func (r *RootResolver) Query() *QueryResolver {
	return &QueryResolver{}
}

func (r *RootResolver) Mutation() *MutationResolver {
	return &MutationResolver{}
}
