package ticker

import (
	"time"

	"agent.brume.dev/internal/config"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type Ticker struct {
	RapidTicker *time.Ticker
	SlowTicker  *time.Ticker
}

func NewTicker(cfg *config.AgentConfig) *Ticker {
	log.Info().Msg("Starting the tickers")
	rapidTicker := time.NewTicker(time.Duration(cfg.RapidTicker) * time.Second)
	slowTicker := time.NewTicker(time.Duration(cfg.SlowTicker) * time.Second)

	return &Ticker{
		RapidTicker: rapidTicker,
		SlowTicker:  slowTicker,
	}
}

var TickerModule = fx.Module("ticker",
	fx.Provide(NewTicker),
	fx.Invoke(func(ticker *Ticker) {
		log.Info().Msg("Ticker started")
	}),
)
