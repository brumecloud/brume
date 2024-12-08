package log

import (
	"context"

	"brume.dev/internal/db"
	log_model "brume.dev/logs/model"
	"github.com/rs/zerolog/log"
)

type LogActivity struct {
	logService *LogService
	chdb       *db.ClickhouseDB
}

func NewLogActivity(logService *LogService, chdb *db.ClickhouseDB) *LogActivity {
	return &LogActivity{logService: logService, chdb: chdb}
}

// once we are in the master, the log are formatted and ready to be ingested
// this where we should inform the chan, if connected
// Logs can come from any type of runner
func (l *LogActivity) IngestLogs(ctx context.Context, logs []*log_model.Log) error {
	log.Info().Uint("logs", uint(len(logs))).Msg("Ingesting logs")

	err := l.chdb.Gorm.Create(logs).Error

	if err != nil {
		log.Error().Err(err).Msg("Error ingesting logs")
		return err
	}

	return nil
}
