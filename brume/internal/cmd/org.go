package cmd

import (
	"fmt"
	"time"

	"github.com/briandowns/spinner"
	"github.com/fatih/color"
	"github.com/manifoldco/promptui"
	"github.com/rodaine/table"
	"github.com/spf13/cobra"
)

func NewOrgCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:          "org",
		Short:        "Manage & visualize yourorganizations",
		SilenceUsage: false,
		Args:         cobra.NoArgs,
	}

	create := &cobra.Command{
		Use:          "create",
		Short:        "Create a new organization",
		RunE:         createOrg(),
		SilenceUsage: true,
		Args:         cobra.NoArgs,
	}

	getUserOrg := &cobra.Command{
		Use:          "list",
		Short:        "List user organization",
		RunE:         getUserOrg(),
		SilenceUsage: true,
		Args:         cobra.NoArgs,
	}

	cmd.AddCommand(create, getUserOrg)

	return cmd
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

func getUserOrg() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		clt := GetBrumeClient()
		token, err := GetToken()

		if err != nil {
			return err
		}

		orgs, err := clt.GetUserOrganizations(token)

		if err != nil {
			return err
		}

		tableHeader := color.New(color.FgHiBlue).SprintfFunc()
		columnFmt := color.New(color.Bold).SprintfFunc()

		tbl := table.New("Name", "Id")
		tbl.WithHeaderFormatter(tableHeader).WithFirstColumnFormatter(columnFmt)

		for _, org := range orgs {
			tbl.AddRow(org.Name, org.Id)
		}

		tbl.Print()

		return nil
	}
}
