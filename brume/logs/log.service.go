package log

import (
	"context"

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

func (l *LogService) GetLogs(ctx context.Context, projectID uuid.UUID) ([]*log_model.Log, error) {
	logger.Trace().Str("projectId", projectID.String()).Msg("Getting logs")

	logs := make([]*log_model.Log, 0)

	return logs, nil
}
