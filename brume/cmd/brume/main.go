package main

import (
	cmd "brume.dev/internal/cmd"
	"os"
)

func main() {
	root := cmd.NewRootCommand()

	// running style
	agent := cmd.NewAgentCmd()
	master := cmd.NewMasterCommand()

	// cli
	project := cmd.NewProjectCmd()
	login := cmd.NewLoginCommand()

	root.AddCommand(project, login, master, agent)

	if err := root.Execute(); err != nil {
		os.Exit(1)
	}
}
