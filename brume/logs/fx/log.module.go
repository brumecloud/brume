package fx_log

import (
	log "brume.dev/logs"
	"go.uber.org/fx"
)

var LogModule = fx.Options(
	fx.Provide(log.NewLogService, log.NewLogActivity),
	fx.Invoke(func(l *log.LogService, logActivity *log.LogActivity) {}),
)
