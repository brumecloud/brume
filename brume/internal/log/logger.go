package log

import (
	"os"
	"strings"
	"time"

	"brume.dev/internal/config"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
	"github.com/rs/zerolog/pkgerrors"
)

func RootLogger() zerolog.Logger {
	zerolog.TimeFieldFormat = time.RFC3339Nano
	zerolog.DurationFieldUnit = time.Nanosecond
	zerolog.ErrorStackMarshaler = pkgerrors.MarshalStack

	return zlog.Output(zerolog.ConsoleWriter{Out: os.Stderr}).Level(zerolog.InfoLevel)
}

// init the logger for the log module
func LogLogger() zerolog.Logger {
	cfg := config.GetConfig()

	level := "warn"

	rootLevel, ok := cfg.Logs["root"].(string)
	if ok {
		level = rootLevel
	}

	// internal.log
	internalLogLevel, ok := cfg.Logs["internal.log"].(string)
	if ok {
		level = internalLogLevel
	}

	parsedLevel, err := zerolog.ParseLevel(level)
	if err != nil {
		parsedLevel = zerolog.WarnLevel
	}

	return RootLogger().Level(parsedLevel).With().Str("module", "internal.log").Logger()
}

var logger = LogLogger()

func diveIntoLogs(module []string, logs map[string]any, tmpLevel string) string {
	if len(module) == 0 {
		return tmpLevel
	}

	if len(module) == 1 {
		// we found an exact match, great
		if level, ok := logs[module[0]].(string); ok {
			return level
		}
		return tmpLevel
	}

	key := module[0]
	if subModule, ok := logs[key].(map[string]any); ok {
		return diveIntoLogs(module[1:], subModule, tmpLevel)
	}

	return tmpLevel
}

// GetLogger returns a logger for a given module
// it will mute certain modules from logging
func GetLogger(module string) zerolog.Logger {
	cfg := config.GetConfig()

	logger.Debug().Str("module", module).Msg("Computing logger level")

	rootLevel, ok := cfg.Logs["root"].(string)
	if !ok {
		rootLevel = "warn"
	}

	splitModule := strings.Split(module, ".")
	level := diveIntoLogs(splitModule, cfg.Logs, rootLevel)

	parsedLevel, err := zerolog.ParseLevel(level)
	if err != nil {
		parsedLevel = zerolog.WarnLevel
	}

	return RootLogger().Level(parsedLevel).With().Str("module", module).Logger()
}
