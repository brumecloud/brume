package cmd

import (
	"fmt"
	"regexp"
	"time"

	"errors"

	"github.com/briandowns/spinner"
	"github.com/manifoldco/promptui"
	"github.com/rs/zerolog/log"
	"github.com/spf13/cobra"
)

func NewLoginCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:           "login",
		Short:         "Login to Brume service",
		RunE:          runLogin(),
		Args:          cobra.NoArgs,
		SilenceErrors: false,
		SilenceUsage:  true,
	}

	return cmd
}

func isEmailValid(e string) bool {
	emailRegex := regexp.MustCompile(`^[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,4}$`)
	return emailRegex.MatchString(e)
}

func runLogin() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		fmt.Println("Login to Brume ☁️")

		methodPrompt := promptui.Select{
			Label:        "Choose a login method",
			Items:        []string{"Email / Password method", "GitHub method", "Google method"},
			HideSelected: true,
		}

		_, method, err := methodPrompt.Run()

		if err != nil {
			return err
		}

		switch method {
		case "Email / Password method":
			return emailPassword()
		case "GitHub method":
			return nil
		case "Google method":
			return nil
		default:
			return errors.New("invalid login method")
		}
	}
}

func emailPassword() error {
	emailPrompt := promptui.Prompt{
		Label: "Email",
		Validate: func(input string) error {
			if !isEmailValid(input) {
				return errors.New("the email is invalid")
			}
			return nil
		},
	}

	passwordPrompt := promptui.Prompt{
		Label: "Password",
		Mask:  '*',
	}

	_email, err := emailPrompt.Run()
	if err != nil {
		return err
	}

	_pass, err := passwordPrompt.Run()

	if err != nil {
		return err
	}

	_ = _email
	_ = _pass

	s := spinner.New(spinner.CharSets[14], 100*time.Millisecond)
	s.Start()
	s.Suffix = " Logging in..."
	// time.Sleep(1 * time.Second)
	token, err := loginUsingClient(_email, _pass)
	s.Stop()

	if err != nil {
		return err
	}

	log.Debug().Str("token", token)

	fmt.Println("Login successful 🎉")
	return nil
}

func loginUsingClient(email string, password string) (string, error) {
	clt := GetBrumeClient()
	token, err := clt.PasswordLogin(email, password)

	if err != nil {
		return "", err
	}

	err = SetToken(token)

	if err != nil {
		return "", err
	}

	return token, nil
}
