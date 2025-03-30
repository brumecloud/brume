package log

import (
	"os"
	"strings"
	"time"

	"github.com/brumecloud/agent/internal/config"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
	"github.com/rs/zerolog/pkgerrors"
)

var cfg *config.GeneralConfig

var logger = RootLogger().With().Str("module", "log").Logger()

func RootLogger() zerolog.Logger {
	if cfg == nil {
		cfg = config.LoadAgentConfig()
	}

	level, err := zerolog.ParseLevel(cfg.Logs.Level)
	if err != nil {
		level = zerolog.DebugLevel
	}

	return zlog.Output(zerolog.ConsoleWriter{Out: os.Stderr}).Level(level)
}

func GetLogger(module string) zerolog.Logger {
	// some filter are set
	if cfg.Logs.Filter != "" {
		if strings.Contains(cfg.Logs.Filter, module) {
			return RootLogger().With().Str("module", module).Logger()
		}

		logger.Warn().Str("module", module).Str("log_filter", cfg.Logs.Filter).Msg("module not in log filters")
		return zerolog.Nop()
	} else {
		// we dont have a log filters
		return RootLogger().With().Str("module", module).Logger()
	}
}

func InitLogger() {
	zerolog.TimeFieldFormat = time.RFC3339Nano
	zerolog.DurationFieldUnit = time.Nanosecond
	zerolog.ErrorStackMarshaler = pkgerrors.MarshalStack

	_ = os.Mkdir("logs", os.ModePerm)

	zlog.Logger = GetLogger("main")
}
