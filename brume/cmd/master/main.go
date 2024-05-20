package main

import "brume.dev/injection"

// entrypoint for the master node of a brume cluster

func main() {
	injector := injection.NewGlobalInjector()
	injector.Run()
}
