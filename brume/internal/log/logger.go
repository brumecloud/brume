package log

import (
	"os"
	"time"

	"brume.dev/internal/config"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
	"github.com/rs/zerolog/pkgerrors"
)

var logger = RootLogger().With().Str("module", "log").Logger()

func RootLogger() zerolog.Logger {
	zerolog.TimeFieldFormat = time.RFC3339Nano
	zerolog.DurationFieldUnit = time.Nanosecond
	zerolog.ErrorStackMarshaler = pkgerrors.MarshalStack

	return zlog.Output(zerolog.ConsoleWriter{Out: os.Stderr}).Level(zerolog.InfoLevel)
}

// GetLogger returns a logger for a given module
// it will mute certain modules from logging
func GetLogger(module string) zerolog.Logger {
	cfg := config.GetConfig()

	// mute certain modules from logging
	if level, ok := cfg.Logs[module]; ok {
		logger.Info().Str("module", module).Str("log_level", level).Msg("Module logger level")
		if level == "silent" {
			return zerolog.Nop()
		}

		level, err := zerolog.ParseLevel(level)
		if err != nil {
			level = zerolog.InfoLevel
		}

		return RootLogger().Level(level).With().Str("module", module).Logger()
	}

	logger.Error().Str("module", module).Msg("module not in log filters")
	return zerolog.Nop()
}
