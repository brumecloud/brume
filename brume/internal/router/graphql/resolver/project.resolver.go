package resolver

import (
	project "brume.dev/project/model"
	"github.com/google/uuid"
)

type ProjectResolver struct {
	p *project.Project
	q *QueryResolver
}

func (q *QueryResolver) Project(args struct{ ID uuid.UUID }) (*ProjectResolver, error) {

	// project, err := q.projectService.GetProjectByID(args.ID)

	return &ProjectResolver{
		p: &project.Project{
			Name:        "simple project test",
			Description: "this is a simple project",
			ID:          args.ID,
		},
		q: q,
	}, nil
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
