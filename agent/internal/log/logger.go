package log

import (
	"os"
	"time"

	"github.com/brumecloud/agent/internal/config"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
	"github.com/rs/zerolog/pkgerrors"
)

var cfg *config.AgentConfig

func GetLogger(module string) zerolog.Logger {
	if cfg == nil {
		cfg = config.LoadAgentConfig()
	}

	level, err := zerolog.ParseLevel(cfg.LogLevel)
	if err != nil {
		level = zerolog.DebugLevel
	}

	return zlog.Output(zerolog.ConsoleWriter{Out: os.Stderr}).Level(level).With().Str("module", module).Logger()
}

func InitLogger() {
	zerolog.TimeFieldFormat = time.RFC3339Nano
	zerolog.DurationFieldUnit = time.Nanosecond
	zerolog.ErrorStackMarshaler = pkgerrors.MarshalStack

	_ = os.Mkdir("logs", os.ModePerm)

	zlog.Logger = GetLogger("main")
}
