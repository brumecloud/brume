package fx_common

import (
	"brume.dev/internal/common"
	"go.uber.org/fx"
)

var CommonModule = fx.Options(
	fx.Provide(common.NewAuthentificationService),
	fx.Invoke(func(s *common.AuthentificationService) {}),
)
