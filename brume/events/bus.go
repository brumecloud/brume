package events

import (
	"go.uber.org/zap"
)

// all the event bus of the app
// org:app:domain is the structure
type GlobalBus struct {
	channels map[string]Channel
}

func NewGlobalBus() *GlobalBus {
	return &GlobalBus{
		channels: make(map[string]Channel),
	}
}

func (g *GlobalBus) Launch(log *zap.Logger) {
	log.Info("Launching global bus with")

	for _, channel := range g.channels {
		log.Debug("Launching channel", zap.String("channel", channel.name))
	}
}

// create a new channel, channel are domain based
func (g *GlobalBus) RegisterChannel(name string) error {
	return nil
}
