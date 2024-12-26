package main

import (
	cmd "brume.dev/internal/cmd"
	"os"
)

func main() {
	root := cmd.NewRootCommand()

	// running style
	master := cmd.NewMasterCommand()

	// cli
	project := cmd.NewProjectCmd()
	login := cmd.NewLoginCommand()

	root.AddCommand(project, login, master)

	if err := root.Execute(); err != nil {
		os.Exit(1)
	}
}
