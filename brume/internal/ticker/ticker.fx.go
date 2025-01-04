package brume_ticker

import (
	"time"

	"brume.dev/internal/config"
	"brume.dev/internal/log"
	"go.uber.org/fx"
)

var logger = log.GetLogger("ticker")

var TickerModule = fx.Module("ticker",
	fx.Provide(NewTickerService),
)

type TickerService struct {
	rapidTicker *time.Ticker
}

func NewTickerService(cfg *config.BrumeConfig) *TickerService {
	logger.Info().Int("rapidTicker", cfg.RapidTicker).Msg("Starting the tickers")

	rapidTicker := time.NewTicker(time.Duration(cfg.RapidTicker) * time.Second)

	return &TickerService{
		rapidTicker: rapidTicker,
	}
}

func (t *TickerService) Start() {
}
