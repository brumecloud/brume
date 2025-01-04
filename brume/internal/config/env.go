package config

import (
	"github.com/rs/zerolog/log"
	"github.com/spf13/viper"
	"go.uber.org/fx"
)

type BrumeConfig struct {
	LogLevel string `mapstructure:"LOG_LEVEL"`

	HTTPPort int    `mapstructure:"HTTP_PORT"`
	GRPCPort int    `mapstructure:"GRPC_PORT"`
	HTTPHost string `mapstructure:"HTTP_HOST"`

	RapidTicker int `mapstructure:"RAPID_TICKER"`

	ClickhouseHost string `mapstructure:"CLICKHOUSE_HOST"`
	ClickhousePort int    `mapstructure:"CLICKHOUSE_PORT"`
	ClickhouseDB   string `mapstructure:"CLICKHOUSE_DB"`

	RedisHost     string `mapstructure:"REDIS_HOST"`
	RedisPort     int    `mapstructure:"REDIS_PORT"`
	RedisDB       int    `mapstructure:"REDIS_DB"`
	RedisPassword string `mapstructure:"REDIS_PASSWORD"`

	TemporalHost string `mapstructure:"TEMPORAL_HOST"`
	TemporalPort int    `mapstructure:"TEMPORAL_PORT"`
}

var logger = log.With().Str("module", "config").Logger()

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
	viper.SetDefault("HTTP_PORT", 9876)
	viper.SetDefault("GRPC_PORT", 9877)
	viper.SetDefault("HTTP_HOST", "0.0.0.0")

	// Ticker config
	// Internal ticker for rapid updates
	viper.SetDefault("RAPID_TICKER", 10)

	// Clickhouse config
	viper.SetDefault("CLICKHOUSE_HOST", "clickhouse")
	viper.SetDefault("CLICKHOUSE_PORT", 9000)
	viper.SetDefault("CLICKHOUSE_DB", "brume")

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
