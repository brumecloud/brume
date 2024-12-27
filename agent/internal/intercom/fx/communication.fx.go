package intercom_fx

import (
	intercom_service "github.com/brumecloud/agent/internal/intercom"
	"go.uber.org/fx"
)

var IntercomModule = fx.Module("intercom",
	fx.Provide(intercom_service.NewIntercomService),
)
