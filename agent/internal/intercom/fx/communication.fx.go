package intercom_fx

import (
	intercom_service "agent.brume.dev/internal/intercom"
	"go.uber.org/fx"
)

var IntercomModule = fx.Module("intercom",
	fx.Provide(intercom_service.NewIntercomService),
)
