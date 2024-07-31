package log

import (
	"os"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/rs/zerolog/pkgerrors"
)

func GetLogger() zerolog.Logger {
	return log.Output(zerolog.ConsoleWriter{Out: os.Stderr})
}

func InitLogger() {
	zerolog.TimeFieldFormat = time.RFC3339Nano
	zerolog.DurationFieldUnit = time.Nanosecond
	zerolog.ErrorStackMarshaler = pkgerrors.MarshalStack

	log.Logger = GetLogger()
}
