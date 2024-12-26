package main

import (
	"agent.brume.dev/internal/injection"
	"github.com/rs/zerolog/log"
)

func main() {
	log.Info().Msg("Starting Brume Agent")

	injector := injection.NewAgentInjector()
	injector.Run()
}
