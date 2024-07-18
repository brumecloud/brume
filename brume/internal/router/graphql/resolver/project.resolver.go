package resolver

import (
	"context"

	project "brume.dev/project/model"
	"github.com/rs/zerolog/log"
)

type ProjectResolver struct {
	p *project.Project
	q *QueryResolver
}

func (q *QueryResolver) GetProjectById(ctx context.Context, args struct {
	ID string
}) (*ProjectResolver, error) {
	project, err := q.projectService.GetProjectByID(args.ID)

	return &ProjectResolver{p: project, q: q}, err
}

func (p *ProjectResolver) Name() string {
	return p.p.Name
}

func (p *ProjectResolver) Description() string {
	return p.p.Description
}

func (p *ProjectResolver) Id() string {
	return p.p.ID.String()
}

func (p *ProjectResolver) Services() ([]*ServiceResolver, error) {
	betterProject, err := p.q.projectService.GetProjectServices(p.p)

	if err != nil {
		return make([]*ServiceResolver, 0), err
	}

	services := make([]*ServiceResolver, len(betterProject.Services))

	for i, s := range betterProject.Services {
		services[i] = &ServiceResolver{s, p.q}
	}

	return services, nil
}

func (m *MutationResolver) CreateProject(ctx context.Context, args *struct {
	Name        string
	Description *string
}) (*ProjectResolver, error) {
	email := ctx.Value("user").(string)
	log.Info().Str("email", email).Str("name", args.Name).Str("description", *args.Description).Msg("Creating project for")

	project, err := m.projectService.CreateProject(args.Name, *args.Description)
	if err != nil {
		return nil, err
	}
	user, err := m.userService.GetUserByEmail(email)
	if err != nil {
		return nil, err
	}

	_, err = m.userService.AddUserProject(user, project)
	if err != nil {
		return nil, err
	}

	return &ProjectResolver{p: project, q: m.q}, nil
}
