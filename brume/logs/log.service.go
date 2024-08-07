package log

import (
	"context"
	"fmt"
	"time"

	"brume.dev/internal/db"
	log_model "brume.dev/logs/model"
)

type LogService struct {
	db *db.DB
}

func NewLogService(db *db.DB) *LogService {
	return &LogService{
		db: db,
	}
}

func (l *LogService) GetDummyLog(ctx context.Context) ([]*log_model.Log, error) {
	lines := make([]*log_model.Log, 40)
	for i := 0; i < 40; i++ {
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

				log_line := &log_model.Log{
					Message:   fmt.Sprintf("hello sub%d %d", i, j),
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
