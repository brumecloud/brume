package config

import (
	"github.com/go-playground/validator/v10"
	"github.com/rs/zerolog/log"
	"github.com/spf13/viper"
	"go.uber.org/fx"
)

type GeneralConfig struct {
	BuilderID    string             `mapstructure:"builder_id" validate:"required"`
	Logs         map[string]string  `mapstructure:"logs" validate:"required"`
}

// avoid circular dependency
var logger = log.With().Str("module", "config").Logger()

var config *GeneralConfig

func GetConfig() *GeneralConfig {
	if config != nil {
		return config
	}

	cfg := &GeneralConfig{}

	viper.SetConfigName("builder")
	viper.AddConfigPath(".")
	viper.AutomaticEnv()

	err := viper.ReadInConfig()
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to read config file")
		panic(err)
	}

	err = viper.Unmarshal(cfg)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to load config")
		panic(err)
	}

	val := validator.New()
	err = val.Struct(cfg)
	if err != nil {
		for _, err := range err.(validator.ValidationErrors) {
			logger.Error().Err(err).Msg("Failed to validate config")
		}
		panic(err)
	}

	config = cfg
	return cfg
}

var ConfigModule = fx.Module("config", fx.Provide(GetConfig), fx.Invoke(func(cfg *GeneralConfig) {}))
