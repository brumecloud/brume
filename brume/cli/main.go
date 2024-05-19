package main

import "brume.dev/injection"

func main() {
	// initialize the global fx injector
	injector := injection.NewGlobalInjector()
	injector.Launch()
}
