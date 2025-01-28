package log

import (
	"context"
	"fmt"
	"time"

	clickhouse "brume.dev/internal/clickhouse"
	job_service "brume.dev/jobs/service"
	log_model "brume.dev/logs/model"
	"github.com/google/uuid"
	redis "github.com/redis/go-redis/v9"
)

type LogService struct {
	chdb       *clickhouse.ClickhouseDB
	redis      *redis.Client
	jobService *job_service.JobService
}

func NewLogService(chdb *clickhouse.ClickhouseDB, redis *redis.Client, jobService *job_service.JobService) *LogService {
	return &LogService{chdb: chdb, redis: redis, jobService: jobService}
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
	logger.Info().Str("projectId", projectID.String()).Msg("Getting logs")

	logs := make([]*log_model.Log, 0)

	// no deployment_id, we get all logs for the project
	err := l.chdb.Gorm.Where(&log_model.Log{ProjectID: projectID}).Order("timestamp asc").Limit(500).Find(&logs).Error

	return logs, err
}

// push directly to clickhouse
func (l *LogService) IngestLogs(logs []*log_model.AgentLogs) error {
	chLogs := make([]*log_model.Log, 0)

	for _, log := range logs {
		timestamp, err := time.Parse(time.RFC3339, log.Timestamp)
		if err != nil {
			timestamp = time.Now()
		}

		jobID, err := uuid.Parse(log.JobID)
		if err != nil {
			logger.Error().Err(err).Str("job_id", log.JobID).Msg("Failed to parse job id")
			continue
		}

		// todo: cache and optimize this
		job, err := l.jobService.GetJob(jobID)
		if err != nil {
			logger.Error().Err(err).Str("job_id", log.JobID).Msg("Failed to get job")
			continue
		}

		// TODO need to find the job id and the deployment id
		chLogs = append(chLogs, &log_model.Log{
			ID:             uuid.New(),
			ServiceID:      job.ServiceID,
			DeploymentID:   job.Deployment.ID,
			DeploymentName: job.Deployment.ServiceName,
			ProjectID:      job.Deployment.ProjectID,
			Message:        log.Message,
			Level:          log.Level,
			Timestamp:      timestamp,
		})
	}

	err := l.chdb.Gorm.Create(&chLogs).Error
	if err != nil {
		return err
	}

	logger.Debug().Uint("logs", uint(len(logs))).Msg("Ingesting logs")

	return nil
}
