package project_cmd

import "github.com/spf13/cobra"

func NewListCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:          "list",
		Short:        "List all your projects",
		SilenceUsage: true,
		Args:         cobra.NoArgs,
	}

	return cmd
}
