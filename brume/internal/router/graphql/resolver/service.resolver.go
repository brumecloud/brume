package resolver

import (
	"context"

	service "brume.dev/service/model"
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
	Name      string
	ProjectId string
}) (*ServiceResolver, error) {
	service, err := m.serviceService.CreateService(args.Name)
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
