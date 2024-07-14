package resolver

import (
	"context"

	user "brume.dev/account/user/model"
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
	betterUser, err := r.q.userService.GetUserProjects(r.u)

	if err != nil {
		return make([]*ProjectResolver, 0), err
	}

	projects := make([]*ProjectResolver, len(betterUser.Projects))

	for i, p := range betterUser.Projects {
		projects[i] = &ProjectResolver{p, r.q}
	}

	return projects, nil
}

func (r *UserResolver) ID() string {
	return r.u.ID.String()
}
