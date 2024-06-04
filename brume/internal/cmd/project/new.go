package project_cmd

import "github.com/spf13/cobra"

func NewNewCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:          "new",
		Short:        "Create a new project",
		SilenceUsage: true,
		Args:         cobra.NoArgs,
	}

	return cmd
}
