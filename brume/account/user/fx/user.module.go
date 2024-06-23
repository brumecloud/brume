package fx_user

import (
	"brume.dev/account/user"
	"go.uber.org/fx"
)

var UserModule = fx.Options(
	fx.Provide(user.NewUserService),
	fx.Invoke(func(s *user.UserService) {}),
)
