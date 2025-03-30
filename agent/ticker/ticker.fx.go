package ticker

import (
	"time"

	"github.com/brumecloud/agent/internal/config"
	"github.com/brumecloud/agent/internal/log"
	"go.uber.org/fx"
)

var logger = log.GetLogger("ticker")

type Ticker struct {
	RapidTicker *time.Ticker
	SlowTicker  *time.Ticker
}

func NewTicker(cfg *config.GeneralConfig) *Ticker {
	logger.Info().Int("rapidTicker", cfg.Orchestrator.RapidTicker).Int("slowTicker", cfg.Orchestrator.SlowTicker).Msg("Starting the tickers")

	rapidTicker := time.NewTicker(time.Duration(cfg.Orchestrator.RapidTicker) * time.Second)
	slowTicker := time.NewTicker(time.Duration(cfg.Orchestrator.SlowTicker) * time.Second)

	return &Ticker{
		RapidTicker: rapidTicker,
		SlowTicker:  slowTicker,
	}
}

func (t *Ticker) Stop() {
	t.RapidTicker.Stop()
	t.SlowTicker.Stop()
}

var TickerModule = fx.Module("ticker",
	fx.Provide(NewTicker),
	fx.Invoke(func(ticker *Ticker) {
		logger.Info().Msg("Ticker started")
	}),
)
