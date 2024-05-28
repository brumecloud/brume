package cmd

import "github.com/spf13/cobra"

func NewRootCommand() *cobra.Command {
	return &cobra.Command{
		Use:   "brume",
		Short: "Brume helps you create your cloud, closer to you",
	}
}
