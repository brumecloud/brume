package log

import (
	"context"

	clickhouse "brume.dev/internal/clickhouse"
	"brume.dev/internal/log"
	log_model "brume.dev/logs/model"
)

var logger = log.GetLogger("log_activity")

type LogActivity struct {
	logService *LogService
	chdb       *clickhouse.ClickhouseDB
}

func NewLogActivity(logService *LogService, chdb *clickhouse.ClickhouseDB) *LogActivity {
	return &LogActivity{logService: logService, chdb: chdb}
}

// once we are in the master, the log are formatted and ready to be ingested
// this where we should inform the chan, if connected
// Logs can come from any type of runner
func (l *LogActivity) IngestLogs(ctx context.Context, logs []*log_model.Log) error {
	logger.Info().Uint("logs", uint(len(logs))).Msg("Ingesting logs")

	err := l.chdb.Gorm.Create(logs).Error
	if err != nil {
		logger.Error().Err(err).Msg("Error ingesting logs")
		return err
	}

	return nil
}
