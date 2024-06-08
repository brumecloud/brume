package fx_org

import (
	"brume.dev/account/org"
	"go.uber.org/fx"
)

var OrgModule = fx.Options(
	fx.Provide(org.NewOrganizationService),
	fx.Invoke(func(s *org.OrganizationService) {}),
)
