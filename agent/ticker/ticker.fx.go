package ticker

import (
	"time"

	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type Ticker struct {
	RapidTicker *time.Ticker
	SlowTicker  *time.Ticker
}

func NewTicker() *Ticker {
	log.Info().Msg("Starting the tickers")
	rapidTicker := time.NewTicker(1 * time.Second)
	slowTicker := time.NewTicker(5 * time.Second)

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
