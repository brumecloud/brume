package fx_http

import (
	http_server "brume.dev/internal/router/http"
	"go.uber.org/fx"
)

var HttpModule = fx.Options(
	fx.Provide(http_server.NewHTTPServer, http_server.NewMonitoringHTTPRouterV1, http_server.NewSchedulerHTTPRouterV1),
	fx.Invoke(func(s *http_server.BrumeHTTPServer, httpRouter *http_server.MonitoringHTTPRouterV1, schedulerHTTPRouter *http_server.SchedulerHTTPRouterV1) {
	}),
)
