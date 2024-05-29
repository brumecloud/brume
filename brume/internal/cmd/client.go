package cmd

import (
	client "github.com/brume/brume/client/v1"
)

func GetBrumeClient() *client.BrumeClient {
	clt, err := client.NewClient(client.BrumeClientConfig{
		Host: "localhost",
		Port: 9876,
	})

	if err != nil {
		panic(err)
	}

	return clt
}
