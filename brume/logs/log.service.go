package log

import (
	"context"
	"fmt"
	"strings"

	clickhouse "brume.dev/internal/clickhouse"
	job_service "brume.dev/jobs/service"
	log_model "brume.dev/logs/model"
	project_service "brume.dev/project"
	"github.com/google/uuid"
	redis "github.com/redis/go-redis/v9"
)

type LogService struct {
	chdb           *clickhouse.ClickhouseDB
	redis          *redis.Client
	jobService     *job_service.JobService
	ProjectService *project_service.ProjectService
}

func NewLogService(chdb *clickhouse.ClickhouseDB, redis *redis.Client, jobService *job_service.JobService, projectService *project_service.ProjectService) *LogService {
	return &LogService{chdb: chdb, redis: redis, jobService: jobService, ProjectService: projectService}
}

// Get the logs for all the services in the project, across all the machines and the differents containers
// we need to cache the project -> container id mapping
// this is easy to burst, as we know when we deploy a new version of a service
func (l *LogService) GetLogs(ctx context.Context, projectID uuid.UUID) ([]*log_model.Log, error) {
	logger.Trace().Str("projectId", projectID.String()).Msg("Getting logs")

	project, err := l.ProjectService.GetProjectByID(projectID)
	if err != nil {
		logger.Error().Err(err).Str("projectId", projectID.String()).Msg("Failed to get project")
		return nil, err
	}

	containersIds := make([]string, 0)

	for _, service := range project.Services {
		// get all the jobs for the service
		jobs, err := l.jobService.GetJobsByServiceID(service.ID)
		if err != nil {
			logger.Error().Err(err).Str("serviceId", service.ID.String()).Msg("Failed to get jobs")
			return nil, err
		}

		for _, job := range jobs {
			if job.ContainerID != nil {
				containersIds = append(containersIds, *job.ContainerID)
			}
		}
	}

	logger.Info().Interface("containersIds", containersIds).Msg("Containers ids")

	if len(containersIds) == 0 {
		return nil, nil
	}

	logs := make([]*log_model.Log, 0)

	query := fmt.Sprintf("SELECT `Timestamp`, `SeverityText`, `LogAttributes` FROM brume.otel_logs WHERE LogAttributes['container_id'] IN (%s)", strings.Join(containersIds, ","))

	rows, err := l.chdb.Conn.Query(context.Background(), query)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to get logs")
		return nil, err
	}

	for rows.Next() {
		var log log_model.Log
		err = rows.Scan(&log.Timestamp, &log.SeverityText, &log.LogAttributes)
		if err != nil {
			logger.Error().Err(err).Msg("Failed to scan log")
		}
		logs = append(logs, &log)
	}

	logger.Info().Interface("logs", logs).Msg("Logs")

	return logs, nil
}
