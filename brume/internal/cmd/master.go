package cmd

import (
	"brume.dev/internal/injection"
	brumelog "brume.dev/internal/log"
	"github.com/rs/zerolog/log"
	"github.com/spf13/cobra"
)

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
		brumelog.InitLogger()
		log.Info().Msg("Brume v0.1 - Master Node")

		injector := injection.NewMasterInjector()
		injector.Run()

		return nil
	}
}
