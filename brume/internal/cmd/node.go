package cmd

import (
	"brume.dev/internal/injection"
	brumelog "brume.dev/internal/log"
	"github.com/rs/zerolog/log"
	"github.com/spf13/cobra"
)

func NewAgentCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "agent",
		Short: "Run & manage your brume agent",
		RunE:  runAgent(),
		Args:  cobra.NoArgs,
	}

	return cmd
}

func runAgent() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		brumelog.InitLogger()
		log.Info().Msg("Brume v0.1 - Agent")

		injector := injection.NewAgentInjector()
		injector.Run()

		return nil
	}
}
