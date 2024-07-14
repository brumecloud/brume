package resolver

import (
	project "brume.dev/project/model"
)

type ProjectResolver struct {
	p *project.Project
	q *QueryResolver
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
