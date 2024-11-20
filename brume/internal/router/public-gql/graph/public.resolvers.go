package public_graph

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.
// Code generated by github.com/99designs/gqlgen version v0.17.49

import (
	"context"
	"time"

	user_model "brume.dev/account/user/model"
	builder_model "brume.dev/builder/model"
	generated "brume.dev/internal/router/public-gql/graph/generated/generated.go"
	public_graph_model "brume.dev/internal/router/public-gql/graph/model"
	log_model "brume.dev/logs/model"
	project_model "brume.dev/project/model"
	runner_model "brume.dev/runner/model"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
)

// ID is the resolver for the id field.
func (r *logResolver) ID(ctx context.Context, obj *log_model.Log) (string, error) {
	return obj.ID.String(), nil
}

// Timestamp is the resolver for the timestamp field.
func (r *logResolver) Timestamp(ctx context.Context, obj *log_model.Log) (string, error) {
	return obj.Timestamp.Format(time.RFC3339), nil
}

// CreateProject is the resolver for the createProject field.
func (r *mutationResolver) CreateProject(ctx context.Context, name string, description *string) (*project_model.Project, error) {
	return r.ProjectService.CreateProject(name, *description)
}

// AddServiceToProject is the resolver for the addServiceToProject field.
func (r *mutationResolver) AddServiceToProject(ctx context.Context, projectID string, input public_graph_model.CreateServiceInput) (*service_model.Service, error) {
	service, err := r.ServiceService.CreateService(input.Name, uuid.MustParse(projectID), input.Image)

	if err != nil {
		return nil, err
	}

	project, perr := r.ProjectService.GetProjectByID(projectID)

	if perr != nil {
		return nil, perr
	}

	_, err = r.ProjectService.AddServiceToProject(project, service)
	return service, err
}

// UpdateBuilder is the resolver for the updateBuilder field.
func (r *mutationResolver) UpdateBuilder(ctx context.Context, serviceID string, data public_graph_model.BuilderDataInput) (*builder_model.Builder, error) {
	builderData := builder_model.BuilderData{
		Image:    data.Image,
		Registry: data.Registry,
		Tag:      data.Tag,
	}

	return r.ServiceService.UpdateBuilder(uuid.MustParse(serviceID), builderData)
}

// UpdateRunner is the resolver for the updateRunner field.
func (r *mutationResolver) UpdateRunner(ctx context.Context, serviceID string, data public_graph_model.RunnerDataInput) (*runner_model.Runner, error) {
	runnerData := runner_model.RunnerData{
		Command:        data.Command,
		HealthCheckURL: data.HealthCheckURL,
		Memory: runner_model.RessourceConstraints{
			Request: data.Memory.Request,
			Limit:   data.Memory.Limit,
		},
		CPU: runner_model.RessourceConstraints{
			Request: data.CPU.Request,
			Limit:   data.CPU.Limit,
		},
		Port:          data.Port,
		PublicDomain:  data.PublicDomain,
		PrivateDomain: data.PrivateDomain,
	}

	return r.ServiceService.UpdateRunner(uuid.MustParse(serviceID), runnerData)
}

// DeployProject is the resolver for the deployProject field.
func (r *mutationResolver) DeployProject(ctx context.Context, projectID string) (*project_model.Project, error) {
	return r.ProjectService.DeployProject(uuid.MustParse(projectID))
}

// ID is the resolver for the id field.
func (r *projectResolver) ID(ctx context.Context, obj *project_model.Project) (string, error) {
	return obj.ID.String(), nil
}

// Services is the resolver for the services field.
func (r *projectResolver) Services(ctx context.Context, obj *project_model.Project) ([]*service_model.Service, error) {
	betterProject, err := r.ProjectService.GetProjectServices(obj)
	return betterProject.Services, err
}

// Me is the resolver for the me field.
func (r *queryResolver) Me(ctx context.Context) (*user_model.User, error) {
	email := ctx.Value("user").(string)
	user, err := r.UserService.GetUserByEmail(email)

	return user, err
}

// GetProjectByID is the resolver for the getProjectById field.
func (r *queryResolver) GetProjectByID(ctx context.Context, id string) (*project_model.Project, error) {
	return r.ProjectService.GetProjectByID(id)
}

// ServiceLogs is the resolver for the serviceLogs field.
func (r *queryResolver) ServiceLogs(ctx context.Context, serviceID string) ([]*log_model.Log, error) {
	return r.LogService.GetDummyLog(ctx)
}

// ID is the resolver for the id field.
func (r *serviceResolver) ID(ctx context.Context, obj *service_model.Service) (string, error) {
	return obj.ID.String(), nil
}

// Builder is the resolver for the builder field.
func (r *serviceResolver) Builder(ctx context.Context, obj *service_model.Service) (*builder_model.Builder, error) {
	return &obj.Builder, nil
}

// Runner is the resolver for the runner field.
func (r *serviceResolver) Runner(ctx context.Context, obj *service_model.Service) (*runner_model.Runner, error) {
	return &obj.Runner, nil
}

// ServiceLogs is the resolver for the serviceLogs field.
func (r *subscriptionResolver) ServiceLogs(ctx context.Context, serviceID string) (<-chan []*log_model.Log, error) {
	return r.LogService.GetDummyLogsSub(ctx)
}

// ID is the resolver for the id field.
func (r *userResolver) ID(ctx context.Context, obj *user_model.User) (string, error) {
	return obj.ID.String(), nil
}

// Projects is the resolver for the projects field.
func (r *userResolver) Projects(ctx context.Context, obj *user_model.User) ([]*project_model.Project, error) {
	betterUser, err := r.UserService.GetUserProjects(obj)

	return betterUser.Projects, err
}

// Log returns generated.LogResolver implementation.
func (r *Resolver) Log() generated.LogResolver { return &logResolver{r} }

// Mutation returns generated.MutationResolver implementation.
func (r *Resolver) Mutation() generated.MutationResolver { return &mutationResolver{r} }

// Project returns generated.ProjectResolver implementation.
func (r *Resolver) Project() generated.ProjectResolver { return &projectResolver{r} }

// Query returns generated.QueryResolver implementation.
func (r *Resolver) Query() generated.QueryResolver { return &queryResolver{r} }

// Service returns generated.ServiceResolver implementation.
func (r *Resolver) Service() generated.ServiceResolver { return &serviceResolver{r} }

// Subscription returns generated.SubscriptionResolver implementation.
func (r *Resolver) Subscription() generated.SubscriptionResolver { return &subscriptionResolver{r} }

// User returns generated.UserResolver implementation.
func (r *Resolver) User() generated.UserResolver { return &userResolver{r} }

type logResolver struct{ *Resolver }
type mutationResolver struct{ *Resolver }
type projectResolver struct{ *Resolver }
type queryResolver struct{ *Resolver }
type serviceResolver struct{ *Resolver }
type subscriptionResolver struct{ *Resolver }
type userResolver struct{ *Resolver }
