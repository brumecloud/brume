package main

import (
	"github.com/brume/brume/injection"
	brumelog "github.com/brume/brume/internal/log"
	"github.com/rs/zerolog/log"
)

func main() {
	brumelog.InitLogger()
	log.Info().Msg("Brume v0.1")

	injector := injection.NewMasterInjector()
	injector.Run()
}
