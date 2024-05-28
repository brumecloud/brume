package main

import (
	"os"

	"github.com/brume/brume/internal/cmd"
)

func main() {
	root := cmd.NewRootCommand()

	login := cmd.NewLoginCommand()
	org := cmd.NewOrgCommand()

	root.AddCommand(login)
	root.AddCommand(org)

	if err := root.Execute(); err != nil {
		os.Exit(1)
	}
}
