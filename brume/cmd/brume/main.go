package main

import (
	"os"

	cmd "brume.dev/internal/cmd"
)

func main() {
	root := cmd.NewRootCommand()

	// running style
	master := cmd.NewMasterCommand()

	root.AddCommand(master)

	if err := root.Execute(); err != nil {
		os.Exit(1)
	}
}
