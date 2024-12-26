package ticker

import (
	"time"

	"go.uber.org/fx"
)

type Ticker struct {
	secondTicker     *time.Ticker
	fiveSecondTicker *time.Ticker
}

func NewTicker() *Ticker {
	secondTicker := time.NewTicker(1 * time.Second)
	fiveSecondTicker := time.NewTicker(5 * time.Second)

	return &Ticker{
		secondTicker:     secondTicker,
		fiveSecondTicker: fiveSecondTicker,
	}
}

var TickerModule = fx.Module("ticker",
	fx.Provide(NewTicker),
)
