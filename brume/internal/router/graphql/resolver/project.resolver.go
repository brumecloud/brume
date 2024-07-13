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
