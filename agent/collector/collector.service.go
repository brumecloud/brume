package collector

import (
	"github.com/brumecloud/agent/internal/log"
	"go.uber.org/fx"
)

type CollectorService struct {
	otelBin *OtelBin
}

var logger = log.GetLogger("collector")

var PATH_TO_BINARY = "/brume/agent/collector/bin/otelcol"

func NewCollectorService(lc fx.Lifecycle) *CollectorService {
	collector := &CollectorService{
		otelBin: NewOtelBin(),
	}

	lc.Append(fx.StartHook(collector.Start))
	lc.Append(fx.StopHook(collector.Stop))

	return collector
}

func (c *CollectorService) Start() {
	if err := c.otelBin.Start(); err != nil {
		logger.Error().Err(err).Msg("Failed to start otel collector")
	}

	logger.Info().Msg("Otel collector started")
}

func (c *CollectorService) Stop() {
	if err := c.otelBin.Stop(); err != nil {
		logger.Error().Err(err).Msg("Failed to stop otel collector")
	}

	logger.Info().Msg("Otel collector stopped")
}
