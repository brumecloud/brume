package log

import (
	"context"
	"fmt"
	"time"

	clickhouse "brume.dev/internal/clickhouse"
	log_model "brume.dev/logs/model"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
)

type LogService struct {
	chdb *clickhouse.ClickhouseDB
}

func NewLogService(chdb *clickhouse.ClickhouseDB) *LogService {
	return &LogService{chdb: chdb}
}

func (l *LogService) GetDummyLog(ctx context.Context) ([]*log_model.Log, error) {
	lines := make([]*log_model.Log, 100)
	for i := 0; i < 100; i++ {
		log_line := &log_model.Log{
			Message:   fmt.Sprintf("hello%d", i),
			Level:     "info",
			Timestamp: time.Now(),
		}
		lines[i] = log_line
	}

	return lines, nil
}

func (l *LogService) GetDummyLogsSub(ctx context.Context) (chan []*log_model.Log, error) {
	c := make(chan []*log_model.Log)

	go func() {
		defer close(c)
		i := 0
		for {

			lines := make([]*log_model.Log, 0)
			for j := 0; j < 1; j++ {
				randomID, err := uuid.NewRandom()
				if err != nil {
					panic(err)
				}

				log_line := &log_model.Log{
					ID:        randomID,
					Message:   fmt.Sprintf("hello sub%d", i),
					Level:     "info",
					Timestamp: time.Now(),
				}

				lines = append(lines, log_line)
			}

			select {
			case <-ctx.Done():
				return
			case c <- lines:

			}
			i++
			time.Sleep(500 * time.Millisecond)
		}
	}()

	return c, nil
}

func (l *LogService) GetLogs(ctx context.Context, projectID uuid.UUID) ([]*log_model.Log, error) {
	log.Info().Str("projectId", projectID.String()).Msg("Getting logs")

	logs := make([]*log_model.Log, 0)

	// no deployment_id, we get all logs for the project
	err := l.chdb.Gorm.Where(&log_model.Log{ProjectID: projectID}).Order("timestamp asc").Limit(500).Find(&logs).Error

	return logs, err
}
