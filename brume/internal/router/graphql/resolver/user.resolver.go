package resolver

import (
	"context"

	user "brume.dev/account/user/model"
)

type UserResolver struct {
	u *user.User
}

func (q *QueryResolver) Me(ctx context.Context) (*UserResolver, error) {
	email := ctx.Value("user").(string)
	user, err := q.userService.GetUserByEmail(email)

	return &UserResolver{
		u: user,
	}, err
}

func (r *UserResolver) Avatar() string {
	return r.u.Avatar
}

func (r *UserResolver) Name() string {
	return r.u.Name
}

func (r *UserResolver) ID() string {
	return r.u.ID.String()
}
