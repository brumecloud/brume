package config

import (
	"brume.dev/internal/log"
	"github.com/go-playground/validator/v10"
	"github.com/spf13/viper"
	"go.uber.org/fx"
)

type BrumeConfig struct {
	LogConfig        *LogConfig        `mapstructure:"log" validate:"required,dive"`
	ServerConfig     *ServerConfig     `mapstructure:"server" validate:"required,dive"`
	TickerConfig     *TickerConfig     `mapstructure:"ticker" validate:"required,dive"`
	ClickhouseConfig *ClickhouseConfig `mapstructure:"clickhouse" validate:"required,dive"`
	RedisConfig      *RedisConfig      `mapstructure:"redis" validate:"required,dive"`
	TemporalConfig   *TemporalConfig   `mapstructure:"temporal" validate:"required,dive"`
	PostgresConfig   *PostgresConfig   `mapstructure:"postgres" validate:"required,dive"`
}

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
	Host     string `mapstructure:"host" validate:"required,hostname"`
	Port     int    `mapstructure:"port" validate:"required,min=1,max=65535"`
	DB       int    `mapstructure:"db" validate:"required,min=1"`
	Password string `mapstructure:"password" validate:"required,min=1"`
}

type ClickhouseConfig struct {
	Host     string `mapstructure:"host" validate:"required,hostname"`
	Port     int    `mapstructure:"port" validate:"required,min=1,max=65535"`
	DB       string `mapstructure:"db" validate:"required,min=1"`
	User     string `mapstructure:"user" validate:"required,min=1"`
	Password string `mapstructure:"password" validate:"required,min=1"`
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
	validate := validator.New()
	err = validate.Struct(cfg)
	if err != nil {
		panic(err)
	}

	logger.Info().Interface("config", cfg).Msg("Loaded config")

	return cfg
}

var ConfigModule = fx.Module("config", fx.Provide(LoadBrumeConfig), fx.Invoke(func(cfg *BrumeConfig) {}))
