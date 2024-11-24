package log

import (
	"os"
	"time"

	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
	"github.com/rs/zerolog/pkgerrors"
	"go.temporal.io/sdk/log"
)

func GetLogger() zerolog.Logger {
	return zlog.Output(zerolog.ConsoleWriter{Out: os.Stderr})
}

func InitLogger() {
	zerolog.TimeFieldFormat = time.RFC3339Nano
	zerolog.DurationFieldUnit = time.Nanosecond
	zerolog.ErrorStackMarshaler = pkgerrors.MarshalStack

	_ = os.Mkdir("logs", os.ModePerm)

	zlog.Logger = GetLogger()
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
