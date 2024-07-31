package log

import (
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

func (l *LogService) GetDummyLogs() (chan []log_model.Log, error) {
	c := make(chan []log_model.Log)
	log.Info().Msg("Getting logs")
	go func() {
		for i := 0; i < 100; i++ {
			lines := make([]log_model.Log, 1)
			log_line := log_model.Log{
				Message:   "helllooo",
				Level:     "info",
				Timestamp: time.Now(),
			}
			lines = append(lines, log_line)
			c <- lines

			time.Sleep(100 * 1000)
		}
	}()
	return c, nil
}
