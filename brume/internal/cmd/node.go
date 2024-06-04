package cmd

import "github.com/spf13/cobra"

func NewNodeCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:          "node",
		Short:        "Run & manage your brume nodes",
		SilenceUsage: true,
		Args:         cobra.NoArgs,
	}

	return cmd
}

func runNode() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		return nil
	}
}
