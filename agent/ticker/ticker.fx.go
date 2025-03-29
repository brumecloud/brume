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

func NewTicker(cfg *config.AgentConfig) *Ticker {
	logger.Info().Int("rapidTicker", cfg.RapidTicker).Int("slowTicker", cfg.SlowTicker).Msg("Starting the tickers")

	rapidTicker := time.NewTicker(time.Duration(cfg.RapidTicker) * time.Second)
	slowTicker := time.NewTicker(time.Duration(cfg.SlowTicker) * time.Second)

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
