package fx_ticker

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
	RapidTicker *time.Ticker
}

func NewTickerService(cfg *config.BrumeConfig) *TickerService {
	logger.Info().Int("rapidTicker", cfg.TickerConfig.RapidTicker).Msg("Starting the tickers")

	rapidTicker := time.NewTicker(time.Duration(cfg.TickerConfig.RapidTicker) * time.Second)

	return &TickerService{
		RapidTicker: rapidTicker,
	}
}

func (t *TickerService) Start() {
}
