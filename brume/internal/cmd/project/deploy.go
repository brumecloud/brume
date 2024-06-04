package project_cmd

import "github.com/spf13/cobra"

func NewDeployCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:          "deploy",
		Short:        "Deploy a project",
		SilenceUsage: true,
		Args:         cobra.NoArgs,
	}

	return cmd
}
