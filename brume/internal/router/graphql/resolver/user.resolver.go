package resolver

import (
	"context"

	user "brume.dev/account/user/model"
	"github.com/rs/zerolog/log"
)

type UserResolver struct {
	u *user.User
	q *QueryResolver
}

func (q *QueryResolver) Me(ctx context.Context) (*UserResolver, error) {
	email := ctx.Value("user").(string)
	user, err := q.userService.GetUserByEmail(email)

	return &UserResolver{
		u: user,
		q: q,
	}, err
}

func (r *UserResolver) Avatar() string {
	return r.u.Avatar
}

func (r *UserResolver) Name() string {
	return r.u.Name
}

func (r *UserResolver) Projects() ([]*ProjectResolver, error) {
	betterUser, _ := r.q.userService.GetUserProjects(r.u)

	projects := make([]*ProjectResolver, len(betterUser.Projects))

	for i, p := range betterUser.Projects {
		projects[i] = &ProjectResolver{p, r.q}
	}

	return projects, nil
}

func (r *UserResolver) ID() string {
	return r.u.ID.String()
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

	return &ProjectResolver{p: project}, nil
}
