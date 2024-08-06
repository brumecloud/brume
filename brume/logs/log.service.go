package log

import (
	"context"
	"fmt"
	"time"

	"brume.dev/internal/db"
	log_model "brume.dev/logs/model"
	"github.com/rs/zerolog/log"
)

type LogService struct {
	db *db.DB
}

func NewLogService(db *db.DB) *LogService {
	return &LogService{
		db: db,
	}
}

func (l *LogService) GetDummyLogs(ctx context.Context) (chan []*log_model.Log, error) {
	c := make(chan []*log_model.Log)
	log.Info().Msg("Getting logs")

	go func() {
		defer close(c)
		for i := 0; i < 100; i++ {
			time.Sleep(100 * time.Millisecond)
			lines := make([]*log_model.Log, 0)
			log_line := &log_model.Log{
				Message:   fmt.Sprintf("hello%d", i),
				Level:     "info",
				Timestamp: time.Now(),
			}

			lines = append(lines, log_line)
			log.Info().Int("i", i).Msg("Sending")

			select {
			case <-ctx.Done():
				return
			case c <- lines:

			}
		}
	}()

	return c, nil
}
