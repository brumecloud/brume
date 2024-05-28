package cmd

import (
	"fmt"
	"time"

	"github.com/briandowns/spinner"
	"github.com/manifoldco/promptui"
	"github.com/spf13/cobra"
)

func NewOrgCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "org",
		Short: "Manage organizations",
		RunE:  runOrg(),
		Args:  cobra.NoArgs,
	}

	create := &cobra.Command{
		Use:   "create",
		Short: "Create a new organization",
		RunE:  createOrg(),
		Args:  cobra.NoArgs,
	}

	cmd.AddCommand(create)

	return cmd
}

func runOrg() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		return nil
	}
}

func createOrg() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		prompt := promptui.Prompt{
			Label: "Organization name",
		}

		name, err := prompt.Run()

		if err != nil {
			return err
		}

		s := spinner.New(spinner.CharSets[14], 100*time.Millisecond)
		s.Start()
		s.Suffix = fmt.Sprintf(" Creating organization %s...", name)
		time.Sleep(2 * time.Second)
		s.Stop()

		return nil
	}
}
