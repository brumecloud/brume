package main

import (
	"os"

	"github.com/brume/brume/internal/cmd"
)

func main() {
	root := cmd.NewRootCommand()

	login := cmd.NewLoginCommand()
	org := cmd.NewOrgCommand()
	master := cmd.NewMasterCommand()

	root.AddCommand(login)
	root.AddCommand(org)
	root.AddCommand(master)

	if err := root.Execute(); err != nil {
		os.Exit(1)
	}
}
