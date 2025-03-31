package main

import (
	"github.com/brumecloud/agent/internal/injection"
	"github.com/brumecloud/agent/internal/log"
)

var logger = log.GetLogger("cmd")

func main() {
	logger.Info().Msg("Starting Brume Agent")

	injector := injection.NewAgentInjector()
	injector.Run()
}
