package fx_collector

import (
	"github.com/brumecloud/agent/collector"
	"go.uber.org/fx"
)

var CollectorModule = fx.Module("collector",
	fx.Provide(collector.NewCollectorService),
	fx.Invoke(func(c *collector.CollectorService) {}),
)
