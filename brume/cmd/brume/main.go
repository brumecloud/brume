package main

import (
	"os"

	"github.com/brume/brume/internal/cmd"
)

func main() {
	root := cmd.NewRootCommand()

	project := cmd.NewProjectCmd()
	node := cmd.NewNodeCmd()
	login := cmd.NewLoginCommand()
	master := cmd.NewMasterCommand()

	root.AddCommand(project, login, master, node)

	if err := root.Execute(); err != nil {
		os.Exit(1)
	}
}
