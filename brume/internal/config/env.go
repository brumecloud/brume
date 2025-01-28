package config

import (
	"fmt"
	"os"

	"brume.dev/internal/log"
	"github.com/go-playground/validator/v10"
	"github.com/spf13/viper"
	"go.uber.org/fx"
)

type LogConfig struct {
	LogLevel   string `mapstructure:"level" validate:"required,oneof=debug info warn error"`
	DBLogLevel string `mapstructure:"db_level" validate:"required,oneof=silent error warn info"`
}

type ServerConfig struct {
	Host             string `mapstructure:"host" validate:"required,ip"`
	OrchestratorPort int    `mapstructure:"orchestrator_port" validate:"required,min=1,max=65535"`
	GraphqlPort      int    `mapstructure:"graphql_port" validate:"required,min=1,max=65535"`
	GRPCPort         int    `mapstructure:"grpc_port" validate:"required,min=1,max=65535"`
}

type TickerConfig struct {
	RapidTicker int `mapstructure:"rapid_ticker" validate:"required,min=1"`
}

type RedisConfig struct {
	Host string `mapstructure:"host" validate:"required,hostname"`
	Port int    `mapstructure:"port" validate:"required,min=1,max=65535"`

	// this is stupid but it's what the library (go playground) expects for 0 values
	DB       *int   `mapstructure:"db" validate:"required"`
	Password string `mapstructure:"password"`
}

type ClickhouseConfig struct {
	Host     string `mapstructure:"host" validate:"required,hostname"`
	Port     int    `mapstructure:"port" validate:"required,min=1,max=65535"`
	DB       string `mapstructure:"db" validate:"required,min=1"`
	User     string `mapstructure:"user" validate:"required,min=1"`
	Password string `mapstructure:"password"`
}

type TemporalConfig struct {
	Host string `mapstructure:"host" validate:"required,hostname"`
	Port int    `mapstructure:"port" validate:"required,min=1,max=65535"`
}

type PostgresConfig struct {
	Host     string `mapstructure:"host" validate:"required,hostname"`
	Port     int    `mapstructure:"port" validate:"required,min=1,max=65535"`
	DB       string `mapstructure:"db" validate:"required,min=1"`
	User     string `mapstructure:"user" validate:"required,min=1"`
	Password string `mapstructure:"password" validate:"required,min=1"`
	MaxIdle  int    `mapstructure:"max_idle" validate:"required,min=1"`
	MaxOpen  int    `mapstructure:"max_open" validate:"required,min=1"`
}
type BrumeConfig struct {
	LogConfig        LogConfig        `mapstructure:"log" validate:"required"`
	ServerConfig     ServerConfig     `mapstructure:"server" validate:"required"`
	TickerConfig     TickerConfig     `mapstructure:"ticker" validate:"required"`
	ClickhouseConfig ClickhouseConfig `mapstructure:"clickhouse" validate:"required"`
	RedisConfig      RedisConfig      `mapstructure:"redis" validate:"required"`
	TemporalConfig   TemporalConfig   `mapstructure:"temporal" validate:"required"`
	PostgresConfig   PostgresConfig   `mapstructure:"postgres" validate:"required"`
}

var logger = log.GetLogger("config")

func LoadBrumeConfig() *BrumeConfig {
	cfg := &BrumeConfig{}

	viper.AutomaticEnv()
	viper.AddConfigPath(".")
	viper.AddConfigPath("/brume/brume")
	viper.SetConfigName("brume")
	viper.SetConfigType("toml")

	err := viper.ReadInConfig()
	if err != nil {
		logger.Error().Err(err).Msg("Failed to read config file")
		panic(err)
	}

	err = viper.Unmarshal(cfg)
	if err != nil {
		panic(err)
	}

	logger.Info().Interface("config", cfg).Msg("Loaded raw config")

	// validate the config
	validate := validator.New(validator.WithRequiredStructEnabled())
	err = validate.Struct(cfg)
	if err != nil {
		for _, err := range err.(validator.ValidationErrors) {
			logger.Error().Err(err).Str("value", fmt.Sprintf("%v", err.Value())).Str("validation_tag", err.Tag()).Str("field", err.Field()).Msg("Failed to validate config")
		}
		os.Exit(1)
	}

	logger.Info().Interface("config", cfg).Msg("Loaded config")

	return cfg
}

var ConfigModule = fx.Module("config", fx.Provide(LoadBrumeConfig), fx.Invoke(func(cfg *BrumeConfig) {}))
