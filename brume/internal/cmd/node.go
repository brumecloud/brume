package cmd

import (
	"brume.dev/internal/injection"
	brumelog "brume.dev/internal/log"
	"github.com/rs/zerolog/log"
	"github.com/spf13/cobra"
)

func NewNodeCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "node",
		Short: "Run & manage your brume nodes",
		RunE:  runNode(),
		Args:  cobra.NoArgs,
	}

	return cmd
}

func runNode() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		brumelog.InitLogger()
		log.Info().Msg("Brume v0.1 - Node")

		injector := injection.NewNodeInjector()
		injector.Run()

		return nil
	}
}
