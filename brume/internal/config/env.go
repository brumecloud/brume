package config

import (
	"fmt"
	"os"

	"github.com/go-playground/validator/v10"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
	"github.com/spf13/viper"
	"go.uber.org/fx"
)

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

type PostgresConfig struct {
	Host     string `mapstructure:"host" validate:"required,hostname"`
	Port     int    `mapstructure:"port" validate:"required,min=1,max=65535"`
	DB       string `mapstructure:"db" validate:"required,min=1"`
	User     string `mapstructure:"user" validate:"required,min=1"`
	Password string `mapstructure:"password" validate:"required,min=1"`
	MaxIdle  int    `mapstructure:"max_idle" validate:"required,min=1"`
	MaxOpen  int    `mapstructure:"max_open" validate:"required,min=1"`
	Logs     string `mapstructure:"logs" validate:"required"`
}

type OrchestratorConfig struct {
	UnhealthyMachineThreshold int   `mapstructure:"unhealthy_machine_threshold" validate:"required,min=1"`
	UnhealthyJobThreshold     int   `mapstructure:"unhealthy_job_threshold" validate:"required,min=1"`
	RescheduleJobs            *bool `mapstructure:"reschedule_jobs" validate:"required,boolean"`
}

type WorkOSConfig struct {
	ClientID     string `mapstructure:"client_id" validate:"required,min=1"`
	ClientSecret string `mapstructure:"client_secret" validate:"required,min=1"`
	ConnectionID string `mapstructure:"connection_id" validate:"required,min=1"`
	RedirectURI  string `mapstructure:"redirect_uri" validate:"required,min=1"`
	CookieSecret string `mapstructure:"cookie_secret" validate:"required,min=1"`
}

type BrumeGeneralConfig struct {
	Frontend       string `mapstructure:"frontend" validate:"required,min=1"`
	IsDev          bool   `mapstructure:"is_dev" validate:"required,boolean"`
	SudoProviderID string `mapstructure:"sudo_provider_id" validate:"required,min=1"`
	StaffOrgID     string `mapstructure:"staff_org_id" validate:"required,min=1"`
	PolicyURL      string `mapstructure:"policy_url" validate:"required,min=1"`
}

type DurableConfig struct {
	DurableName     string `mapstructure:"durable_name" validate:"required,min=1"`
	DatabaseConn    string `mapstructure:"database_conn" validate:"required,min=1"`
	DatabaseName    string `mapstructure:"database_name" validate:"required,min=1"`
	AdminServer     bool   `mapstructure:"admin_server" validate:"required,boolean"`
	AdminServerPort int    `mapstructure:"admin_server_port" validate:"required,min=1,max=65535"`
}

type BrumeConfig struct {
	Logs               map[string]string  `mapstructure:"logs" validate:"required"`
	ServerConfig       ServerConfig       `mapstructure:"server" validate:"required"`
	TickerConfig       TickerConfig       `mapstructure:"ticker" validate:"required"`
	ClickhouseConfig   ClickhouseConfig   `mapstructure:"clickhouse" validate:"required"`
	RedisConfig        RedisConfig        `mapstructure:"redis" validate:"required"`
	PostgresConfig     PostgresConfig     `mapstructure:"postgres" validate:"required"`
	OrchestratorConfig OrchestratorConfig `mapstructure:"orchestrator" validate:"required"`
	WorkOSConfig       WorkOSConfig       `mapstructure:"workos" validate:"required"`
	BrumeGeneralConfig BrumeGeneralConfig `mapstructure:"brume" validate:"required"`
	DurableConfig      DurableConfig      `mapstructure:"durable" validate:"required"`
}

// we want to avoid import cycle
// this is the only logger using directly the zerolog package
var logger = zlog.Output(zerolog.ConsoleWriter{Out: os.Stderr}).Level(zerolog.DebugLevel)

var Config *BrumeConfig

func GetConfig() *BrumeConfig {
	if Config != nil {
		return Config
	}

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

	// validate the config
	validate := validator.New(validator.WithRequiredStructEnabled())
	err = validate.Struct(cfg)
	if err != nil {
		for _, err := range err.(validator.ValidationErrors) {
			logger.Error().Err(err).Str("value", fmt.Sprintf("%v", err.Value())).Str("validation_tag", err.Tag()).Str("field", err.Field()).Msg("Failed to validate config")
		}
		os.Exit(1)
	}

	Config = cfg
	return cfg
}

var ConfigModule = fx.Module("config", fx.Provide(GetConfig), fx.Invoke(func(cfg *BrumeConfig) {}))
