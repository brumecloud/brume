package resolver

import (
	user "brume.dev/account/user/model"
	"github.com/google/uuid"
)

type UserResolver struct {
	u *user.User
}

func (*QueryResolver) Me() (*UserResolver, error) {
	return &UserResolver{
		u: &user.User{
			ID:   uuid.MustParse("70fd7b3a-8e44-47e0-b5d0-5c1b08e43ae6"),
			Name: "Paul Planchon",
		},
	}, nil
}

func (r *UserResolver) Avatar() string {
	return string("https://avatars.githubusercontent.com/u/34143515?v=4")
}

func (r *UserResolver) Name() string {
	return r.u.Name
}

func (r *UserResolver) ID() string {
	return r.u.ID.String()
}
