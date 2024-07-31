package resolver

import (
	"context"

	service "brume.dev/service/model"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
)

type ServiceResolver struct {
	s *service.Service
	q *QueryResolver
}

func (s *ServiceResolver) Name() string {
	return s.s.Name
}

func (s *ServiceResolver) Id() string {
	return s.s.ID.String()
}

func (m *MutationResolver) AddServiceToProject(ctx context.Context, args struct {
	ProjectId string
	Input     struct {
		Name  string
		Image string
	}
}) (*ServiceResolver, error) {
	log.Info().Str("project ID", args.ProjectId).Str("service name", args.Input.Name).Str("docker image", args.Input.Image).Msg("Creating a new service")

	service, err := m.serviceService.CreateService(args.Input.Name, uuid.MustParse(args.ProjectId), args.Input.Image)

	if err != nil {
		return nil, err
	}

	project, perr := m.projectService.GetProjectByID(args.ProjectId)

	if perr != nil {
		return nil, perr
	}

	_, err = m.projectService.AddServiceToProject(project, service)

	return &ServiceResolver{s: service}, err
}

func (s *SubscriptionResolver) ServiceLogs(ctx context.Context, args struct {
	ServiceId string
}) (chan []*LogResolver, error) {
	log.Info().Msg("Getting logs for service")
	c := make(chan []*LogResolver)
	go func() {
		logChan, _ := s.logService.GetDummyLogs()
		for logs := range logChan {
			lr := make([]*LogResolver, 5)

			for _, log := range logs {
				resolver := &LogResolver{
					l: &log,
				}
				lr = append(lr, resolver)
			}

			c <- lr
		}
	}()

	return c, nil
}
