package resolver

import (
	"context"

	user "brume.dev/account/user/model"
	"github.com/google/uuid"
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

func (r *UserResolver) Projects() []*ProjectResolver {
	projectResolved, err := r.q.Project(struct{ ID uuid.UUID }{ID: uuid.New()})

	if err != nil {
		return nil
	}

	return []*ProjectResolver{projectResolved}
}

func (r *UserResolver) ID() string {
	return r.u.ID.String()
}
