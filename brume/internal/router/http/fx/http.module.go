package fx_http

import (
	http_server "brume.dev/internal/router/http"
	"go.uber.org/fx"
)

var HttpModule = fx.Options(
	fx.Provide(http_server.NewHTTPServer),
	fx.Invoke(func(s *http_server.BrumeHTTPServer) {}),
)
