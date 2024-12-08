package public_graph

// This file will be automatically regenerated based on the schema, any resolver implementations
// will be copied through when generating and any unknown code will be moved to the end.
// Code generated by github.com/99designs/gqlgen version v0.17.57

import (
	"context"
	"fmt"
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
func (r *deploymentResolver) ID(ctx context.Context, obj *service_model.Deployment) (string, error) {
	return obj.ID.String(), nil
}

// Author is the resolver for the author field.
func (r *deploymentResolver) Author(ctx context.Context, obj *service_model.Deployment) (*user_model.User, error) {
	panic(fmt.Errorf("not implemented: Author - author"))
}

// Logs is the resolver for the logs field.
func (r *deploymentResolver) Logs(ctx context.Context, obj *service_model.Deployment) (*service_model.DeploymentLog, error) {
	return &obj.DeployLog, nil
}

// CreatedAt is the resolver for the createdAt field.
func (r *deploymentResolver) CreatedAt(ctx context.Context, obj *service_model.Deployment) (string, error) {
	return obj.CreatedAt.Format(time.RFC3339), nil
}

// Status is the resolver for the status field.
func (r *deploymentLogResolver) Status(ctx context.Context, obj *service_model.DeploymentLog) (string, error) {
	return string(obj.Status), nil
}

// Duration is the resolver for the duration field.
func (r *deploymentLogResolver) Duration(ctx context.Context, obj *service_model.DeploymentLog) (string, error) {
	duration := obj.Duration.Seconds()
	return fmt.Sprintf("%d", int(duration)), nil
}

// Date is the resolver for the date field.
func (r *deploymentLogResolver) Date(ctx context.Context, obj *service_model.DeploymentLog) (string, error) {
	return obj.Date.Format(time.RFC3339), nil
}

// Type is the resolver for the type field.
func (r *deploymentSourceResolver) Type(ctx context.Context, obj *service_model.DeploymentSource) (string, error) {
	return string(obj.Type), nil
}

// ID is the resolver for the id field.
func (r *logResolver) ID(ctx context.Context, obj *log_model.Log) (string, error) {
	return obj.ID.String(), nil
}

// Timestamp is the resolver for the timestamp field.
func (r *logResolver) Timestamp(ctx context.Context, obj *log_model.Log) (string, error) {
	return obj.Timestamp.Format(time.RFC3339), nil
}

// ServiceID is the resolver for the serviceId field.
func (r *logResolver) ServiceID(ctx context.Context, obj *log_model.Log) (string, error) {
	return obj.ServiceID.String(), nil
}

// DeploymentID is the resolver for the deploymentId field.
func (r *logResolver) DeploymentID(ctx context.Context, obj *log_model.Log) (string, error) {
	return obj.DeploymentID.String(), nil
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

	project, perr := r.ProjectService.GetProjectByID(uuid.MustParse(projectID))

	if perr != nil {
		return nil, perr
	}

	_, err = r.ProjectService.AddServiceToProject(project, service)
	return service, err
}

// DeleteService is the resolver for the deleteService field.
func (r *mutationResolver) DeleteService(ctx context.Context, serviceID string) (*service_model.Service, error) {
	service_uuid, err := uuid.Parse(serviceID)

	if err != nil {
		return nil, err
	}

	return r.ServiceService.DeleteService(service_uuid)
}

// UpdateServiceSettings is the resolver for the updateServiceSettings field.
func (r *mutationResolver) UpdateServiceSettings(ctx context.Context, serviceID string, input public_graph_model.ServiceSettingsInput) (*service_model.Service, error) {
	service_uuid, err := uuid.Parse(serviceID)

	if err != nil {
		return nil, err
	}

	return r.ServiceService.UpdateServiceSettings(service_uuid, input.Name)
}

// UpdateBuilder is the resolver for the updateBuilder field.
func (r *mutationResolver) UpdateBuilder(ctx context.Context, serviceID string, data public_graph_model.BuilderDataInput) (*builder_model.Builder, error) {
	service_uuid, err := uuid.Parse(serviceID)

	if err != nil {
		return nil, err
	}

	builderData := builder_model.BuilderData{
		Image:    data.Image,
		Registry: data.Registry,
		Tag:      data.Tag,
	}

	return r.ServiceService.UpdateBuilder(service_uuid, builderData)
}

// UpdateRunner is the resolver for the updateRunner field.
func (r *mutationResolver) UpdateRunner(ctx context.Context, serviceID string, data public_graph_model.RunnerDataInput) (*runner_model.Runner, error) {
	service_uuid, err := uuid.Parse(serviceID)

	if err != nil {
		return nil, err
	}

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

	return r.ServiceService.UpdateRunner(service_uuid, runnerData)
}

// DeleteDraft is the resolver for the deleteDraft field.
func (r *mutationResolver) DeleteDraft(ctx context.Context, projectID string) (*project_model.Project, error) {
	project_uuid, err := uuid.Parse(projectID)

	if err != nil {
		return nil, err
	}

	return r.ProjectService.DeleteDraft(project_uuid)
}

// DeployProject is the resolver for the deployProject field.
func (r *mutationResolver) DeployProject(ctx context.Context, projectID string) (*project_model.Project, error) {
	project_uuid, err := uuid.Parse(projectID)

	if err != nil {
		return nil, err
	}

	return r.ProjectService.DeployProject(project_uuid)
}

// ID is the resolver for the id field.
func (r *projectResolver) ID(ctx context.Context, obj *project_model.Project) (string, error) {
	return obj.ID.String(), nil
}

// IsDirty is the resolver for the isDirty field.
func (r *projectResolver) IsDirty(ctx context.Context, obj *project_model.Project) (bool, error) {
	return r.ProjectService.IsDirty(obj)
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
	project_uuid, err := uuid.Parse(id)

	if err != nil {
		return nil, err
	}

	return r.ProjectService.GetProjectByID(project_uuid)
}

// ProjectLogs is the resolver for the projectLogs field.
func (r *queryResolver) ProjectLogs(ctx context.Context, projectID string) ([]*log_model.Log, error) {
	return r.LogService.GetLogs(ctx, uuid.MustParse(projectID))
}

// ServiceLogs is the resolver for the serviceLogs field.
func (r *queryResolver) ServiceLogs(ctx context.Context, serviceID string) ([]*log_model.Log, error) {
	return r.LogService.GetDummyLog(ctx)
}

// ID is the resolver for the id field.
func (r *serviceResolver) ID(ctx context.Context, obj *service_model.Service) (string, error) {
	return obj.ID.String(), nil
}

// DraftBuilder is the resolver for the draftBuilder field.
func (r *serviceResolver) DraftBuilder(ctx context.Context, obj *service_model.Service) (*builder_model.Builder, error) {
	return obj.DraftBuilder, nil
}

// DraftRunner is the resolver for the draftRunner field.
func (r *serviceResolver) DraftRunner(ctx context.Context, obj *service_model.Service) (*runner_model.Runner, error) {
	return obj.DraftRunner, nil
}

// Deployments is the resolver for the deployments field.
func (r *serviceResolver) Deployments(ctx context.Context, obj *service_model.Service) ([]*service_model.Deployment, error) {
	return r.ServiceService.GetServiceDeployments(obj.ID)
}

// ServiceLogs is the resolver for the serviceLogs field.
func (r *subscriptionResolver) ServiceLogs(ctx context.Context, serviceID string) (<-chan []*log_model.Log, error) {
	return r.LogService.GetDummyLogsSub(ctx)
}

// ProjectLogs is the resolver for the projectLogs field.
func (r *subscriptionResolver) ProjectLogs(ctx context.Context, projectID string) (<-chan []*log_model.Log, error) {
	panic(fmt.Errorf("not implemented: ProjectLogs - projectLogs"))
}

// ServiceEvents is the resolver for the serviceEvents field.
func (r *subscriptionResolver) ServiceEvents(ctx context.Context, serviceID string) (<-chan []*public_graph_model.ServiceEvent, error) {
	panic(fmt.Errorf("not implemented: ServiceEvents - serviceEvents"))
}

// ID is the resolver for the id field.
func (r *userResolver) ID(ctx context.Context, obj *user_model.User) (string, error) {
	return obj.ID.String(), nil
}

// Projects is the resolver for the projects field.
func (r *userResolver) Projects(ctx context.Context, obj *user_model.User) ([]*project_model.Project, error) {
	projects, err := r.UserService.GetUserProjects(obj)

	return projects, err
}

// Deployment returns generated.DeploymentResolver implementation.
func (r *Resolver) Deployment() generated.DeploymentResolver { return &deploymentResolver{r} }

// DeploymentLog returns generated.DeploymentLogResolver implementation.
func (r *Resolver) DeploymentLog() generated.DeploymentLogResolver { return &deploymentLogResolver{r} }

// DeploymentSource returns generated.DeploymentSourceResolver implementation.
func (r *Resolver) DeploymentSource() generated.DeploymentSourceResolver {
	return &deploymentSourceResolver{r}
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

type deploymentResolver struct{ *Resolver }
type deploymentLogResolver struct{ *Resolver }
type deploymentSourceResolver struct{ *Resolver }
type logResolver struct{ *Resolver }
type mutationResolver struct{ *Resolver }
type projectResolver struct{ *Resolver }
type queryResolver struct{ *Resolver }
type serviceResolver struct{ *Resolver }
type subscriptionResolver struct{ *Resolver }
type userResolver struct{ *Resolver }
