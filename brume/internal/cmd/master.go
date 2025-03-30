package cmd

import (
	"brume.dev/internal/injection"
	"brume.dev/internal/log"
	"github.com/spf13/cobra"
)

var logger = log.GetLogger("main")

func NewMasterCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "master",
		Short: "Run the Brume master node",
		RunE:  runMaster(),
		Args:  cobra.NoArgs,
	}

	return cmd
}

func runMaster() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		logger.Info().Msg("Brume v0.1 - Master Node")

		injector := injection.NewMasterInjector()
		injector.Run()

		return nil
	}
}
