package main

import (
	"brume.dev/injection"
	brumelog "brume.dev/internal/log"
	"github.com/rs/zerolog/log"
)

func main() {
	brumelog.InitLogger()
	log.Info().Msg("Brume v0.1")

	injector := injection.NewMasterInjector()
	injector.Run()
}
