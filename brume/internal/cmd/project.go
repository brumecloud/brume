package cmd

import (
	project_cmd "github.com/brume/brume/internal/cmd/project"
	"github.com/spf13/cobra"
)

func NewProjectCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "project",
		Short: "Manage & visualize your projects",
		Args:  cobra.NoArgs,
	}

	deploy := project_cmd.NewDeployCmd()
	new := project_cmd.NewNewCmd()
	list := project_cmd.NewListCmd()

	cmd.AddCommand(deploy, new, list)

	return cmd
}
