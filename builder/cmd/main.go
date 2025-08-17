package cmd

import (
	"github.com/brumecloud/builder/internal/injection"
	"github.com/brumecloud/builder/internal/log"
)

var logger = log.GetLogger("cmd")

func main() {
	logger.Info().Msg("Starting Brume Builder")

	injector := injection.NewBuilderInjector()
	injector.Run()
}
