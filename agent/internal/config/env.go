package config

import (
	"github.com/rs/zerolog/log"
	"github.com/spf13/viper"
	"go.uber.org/fx"
)

type AgentConfig struct {
	OrchestratorURL string `mapstructure:"ORCHESTRATOR_URL"`
	RapidTicker     int    `mapstructure:"RAPID_TICKER"`
	SlowTicker      int    `mapstructure:"SLOW_TICKER"`
	RetryMax        int    `mapstructure:"RETRY_MAX"`
	Env             string `mapstructure:"ENV"`
	AgentID         string `mapstructure:"AGENT_ID"`
}

var logger = log.With().Str("module", "config").Logger()

func LoadAgentConfig() *AgentConfig {
	cfg := &AgentConfig{}

	SetDefaultConfig()

	viper.AutomaticEnv()
	viper.SetConfigFile(".env")

	err := viper.ReadInConfig()

	if err != nil {
		logger.Warn().Err(err).Msg("Failed to read config file")
		logger.Info().Msg("Using the default values for the agent config")
	}

	err = viper.Unmarshal(cfg)

	if err != nil {
		logger.Error().Err(err).Msg("Failed to load config")
		panic(err)
	}

	return cfg
}

func SetDefaultConfig() {
	viper.SetDefault("ORCHESTRATOR_URL", "http://orchestrator:9876/monitoring/v1")
	viper.SetDefault("RAPID_TICKER", 2)
	viper.SetDefault("SLOW_TICKER", 5)
	viper.SetDefault("ENV", "dev")
	viper.SetDefault("AGENT_ID", "test-agent-123")
	// infinite retries
	viper.SetDefault("RETRY_MAX", 0)
}

var ConfigModule = fx.Module("config", fx.Provide(LoadAgentConfig), fx.Invoke(func(cfg *AgentConfig) {}))
