package log

import (
	"os"
	"strings"
	"time"

	"brume.dev/internal/config"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
	"github.com/rs/zerolog/pkgerrors"
	"go.temporal.io/sdk/log"
)

var cfg *config.BrumeConfig

var logger = RootLogger().With().Str("module", "log").Logger()

func RootLogger() zerolog.Logger {
	zerolog.TimeFieldFormat = time.RFC3339Nano
	zerolog.DurationFieldUnit = time.Nanosecond
	zerolog.ErrorStackMarshaler = pkgerrors.MarshalStack

	if cfg == nil {
		cfg = config.LoadBrumeConfig()
	}

	level, err := zerolog.ParseLevel(cfg.LogConfig.LogLevel)
	if err != nil {
		level = zerolog.DebugLevel
	}

	return zlog.Output(zerolog.ConsoleWriter{Out: os.Stderr}).Level(level)
}

func GetLogger(module string) zerolog.Logger {
	// mute certain modules from logging
	if strings.Contains(cfg.LogConfig.MutedModules, module) {
		return zerolog.Nop()
	}

	// allow certain modules to log
	if cfg.LogConfig.AllowedModules != "*" {
		if strings.Contains(cfg.LogConfig.AllowedModules, module) {
			return RootLogger().With().Str("module", module).Logger()
		}

		logger.Warn().Str("module", module).Str("allowed_modules", cfg.LogConfig.AllowedModules).Str("muted_modules", cfg.LogConfig.MutedModules).Msg("module not in log filters")
		return zerolog.Nop()
	} else {
		// we dont have a log filters
		return RootLogger().With().Str("module", module).Logger()
	}
}

// TemporalZeroLogger implements temporal's log.Logger interface using zerolog
type TemporalZeroLogger struct {
	zl zerolog.Logger
}

// Debug logs message at debug level
func (l *TemporalZeroLogger) Debug(msg string, keyvals ...interface{}) {
	l.zl.Debug().Fields(keyValToFields(keyvals)).Msg(msg)
}

// Info logs message at info level
func (l *TemporalZeroLogger) Info(msg string, keyvals ...interface{}) {
	l.zl.Info().Fields(keyValToFields(keyvals)).Msg(msg)
}

// Warn logs message at warn level
func (l *TemporalZeroLogger) Warn(msg string, keyvals ...interface{}) {
	l.zl.Warn().Fields(keyValToFields(keyvals)).Msg(msg)
}

// Error logs message at error level
func (l *TemporalZeroLogger) Error(msg string, keyvals ...interface{}) {
	l.zl.Error().Fields(keyValToFields(keyvals)).Msg(msg)
}

// keyValToFields converts Temporal's key-value pairs to zerolog fields
func keyValToFields(keyvals []interface{}) map[string]interface{} {
	fields := make(map[string]interface{})
	for i := 0; i < len(keyvals); i += 2 {
		if i+1 < len(keyvals) {
			fields[keyvals[i].(string)] = keyvals[i+1]
		}
	}
	return fields
}

// NewTemporalZeroLogger creates a new TemporalZeroLogger
func NewTemporalZeroLogger(zl zerolog.Logger) log.Logger {
	return &TemporalZeroLogger{zl: zl}
}
