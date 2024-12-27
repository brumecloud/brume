package config

import (
	"github.com/rs/zerolog/log"
	"github.com/spf13/viper"
	"go.uber.org/fx"
)

type AgentConfig struct {
	OrchestratorURL string `mapstructure:"ORCHESTRATOR_URL" default:"http://localhost:8080"`
	RapidTicker     int    `mapstructure:"RAPID_TICKER" default:"2"`
	SlowTicker      int    `mapstructure:"SLOW_TICKER" default:"5"`
	Env             string `mapstructure:"ENV" default:"dev"`
}

func LoadAgentConfig() *AgentConfig {
	cfg := &AgentConfig{}

	viper.AutomaticEnv()
	viper.SetConfigFile(".env")

	err := viper.ReadInConfig()

	if err != nil {
		log.Error().Err(err).Msg("Failed to read config file")
		panic(err)
	}

	err = viper.Unmarshal(cfg)

	if err != nil {
		log.Error().Err(err).Msg("Failed to load config")
		panic(err)
	}

	return cfg
}

var ConfigModule = fx.Module("config", fx.Provide(LoadAgentConfig), fx.Invoke(func(cfg *AgentConfig) {}))
