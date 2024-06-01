package cmd

import (
	"github.com/brume/brume/internal/injection"
	brumelog "github.com/brume/brume/internal/log"
	"github.com/rs/zerolog/log"
	"github.com/spf13/cobra"
)

func NewMasterCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "master",
		Short: "Configure the Brume master node",
		RunE:  runMaster(),
		Args:  cobra.NoArgs,
	}

	return cmd
}

func runMaster() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		brumelog.InitLogger()
		log.Info().Msg("Brume v0.1")

		injector := injection.NewMasterInjector()
		injector.Run()

		return nil
	}
}
