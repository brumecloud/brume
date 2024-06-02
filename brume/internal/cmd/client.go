package cmd

import (
	"fmt"
	"os"
	"path/filepath"

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

func GetToken() (string, error) {
	dirname, err := os.UserHomeDir()

	if err != nil {
		fmt.Println("Failed to get user home directory")
		return "", err
	}

	brumePath := filepath.Join(dirname, ".brume")
	tokenPath := filepath.Join(brumePath, "creds")

	token, err := os.ReadFile(tokenPath)

	if err != nil {
		fmt.Printf("Failed to read token from file (%s) \n", tokenPath)
		return "", err
	}

	return string(token), nil
}

func SetToken(token string) error {
	dirname, err := os.UserHomeDir()

	if err != nil {
		fmt.Println("Failed to get user home directory")
		return err
	}

	brumePath := filepath.Join(dirname, ".brume")

	if _, err := os.Stat(brumePath); os.IsNotExist(err) {
		os.Mkdir(brumePath, 0755)
	}

	tokenPath := filepath.Join(brumePath, "creds")

	// write token to file
	err = os.WriteFile(tokenPath, []byte(token), 0644)

	if err != nil {
		fmt.Printf("Failed to write token to file (%s) \n", tokenPath)
		return err
	}

	return nil
}
