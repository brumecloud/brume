package config

import (
	"brume.dev/internal/log"
	"github.com/spf13/viper"
	"go.uber.org/fx"
)

type BrumeConfig struct {
	LogLevel string `mapstructure:"LOG_LEVEL"`

	GraphqlPort      int    `mapstructure:"GRAPHQL_PORT"`
	OrchestratorPort int    `mapstructure:"ORCHESTRATOR_PORT"`
	Host             string `mapstructure:"HOST"`

	GRPCPort int `mapstructure:"GRPC_PORT"`

	RapidTicker int `mapstructure:"RAPID_TICKER"`

	ClickhouseHost     string `mapstructure:"CLICKHOUSE_HOST"`
	ClickhousePort     int    `mapstructure:"CLICKHOUSE_PORT"`
	ClickhouseDB       string `mapstructure:"CLICKHOUSE_DB"`
	ClickhouseUser     string `mapstructure:"CLICKHOUSE_USER"`
	ClickhousePassword string `mapstructure:"CLICKHOUSE_PASSWORD"`

	RedisHost     string `mapstructure:"REDIS_HOST"`
	RedisPort     int    `mapstructure:"REDIS_PORT"`
	RedisDB       int    `mapstructure:"REDIS_DB"`
	RedisPassword string `mapstructure:"REDIS_PASSWORD"`

	TemporalHost string `mapstructure:"TEMPORAL_HOST"`
	TemporalPort int    `mapstructure:"TEMPORAL_PORT"`
}

var logger = log.GetLogger("config")

func LoadBrumeConfig() *BrumeConfig {
	cfg := &BrumeConfig{}

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
	viper.SetDefault("LOG_LEVEL", "info")

	// HTTP server config
	viper.SetDefault("GRAPHQL_PORT", 9877)
	viper.SetDefault("ORCHESTRATOR_PORT", 9876)
	viper.SetDefault("GRPC_PORT", 9879)
	viper.SetDefault("HOST", "0.0.0.0")

	// Ticker config
	// Internal ticker for rapid updates
	viper.SetDefault("RAPID_TICKER", 10)

	// Clickhouse config
	viper.SetDefault("CLICKHOUSE_HOST", "clickhouse")
	viper.SetDefault("CLICKHOUSE_PORT", 9000)
	viper.SetDefault("CLICKHOUSE_DB", "brume")
	viper.SetDefault("CLICKHOUSE_USER", "brume")
	viper.SetDefault("CLICKHOUSE_PASSWORD", "brumepass")

	// Redis config
	viper.SetDefault("REDIS_HOST", "redis")
	viper.SetDefault("REDIS_PORT", 6379)
	viper.SetDefault("REDIS_DB", 0)
	viper.SetDefault("REDIS_PASSWORD", "")

	// Temporal config
	viper.SetDefault("TEMPORAL_HOST", "temporal")
	viper.SetDefault("TEMPORAL_PORT", 7233)
}

var ConfigModule = fx.Module("config", fx.Provide(LoadBrumeConfig), fx.Invoke(func(cfg *BrumeConfig) {}))
