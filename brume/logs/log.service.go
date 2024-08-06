package log

import (
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

func (l *LogService) GetDummyLogs() (chan []*log_model.Log, error) {
	c := make(chan []*log_model.Log)
	log.Info().Msg("Getting logs")
	go func() {
		for i := 0; i < 10; i++ {
			lines := make([]*log_model.Log, 0)
			log_line := &log_model.Log{
				Message:   fmt.Sprintf("hello%d", i),
				Level:     "info",
				Timestamp: time.Now(),
			}
			lines = append(lines, log_line)
			c <- lines

			time.Sleep(100 * time.Millisecond)
		}
		close(c)
	}()
	return c, nil
}
