package log

import (
	"context"
	"time"

	deployment_service "brume.dev/deployment"
	clickhouse "brume.dev/internal/clickhouse"
	job_service "brume.dev/jobs/service"
	log_model "brume.dev/logs/model"
	project_service "brume.dev/project"
	"github.com/google/uuid"
	redis "github.com/redis/go-redis/v9"
)

type LogService struct {
	chdb              *clickhouse.ClickhouseDB
	redis             *redis.Client
	jobService        *job_service.JobService
	ProjectService    *project_service.ProjectService
	DeploymentService *deployment_service.DeploymentService
}

func NewLogService(chdb *clickhouse.ClickhouseDB, redis *redis.Client, jobService *job_service.JobService, projectService *project_service.ProjectService, deploymentService *deployment_service.DeploymentService) *LogService {
	return &LogService{chdb: chdb, redis: redis, jobService: jobService, ProjectService: projectService, DeploymentService: deploymentService}
}

// Get the logs for all the services in the project, across all the machines and the differents containers
// we need to cache the project -> container id mapping
// this is easy to burst, as we know when we deploy a new version of a service
func (l *LogService) GetLogs(ctx context.Context, projectID uuid.UUID, timestamp time.Time, limit int) ([]*log_model.Log, error) {
	logger.Trace().Str("projectId", projectID.String()).Msg("Getting logs")

	project, err := l.ProjectService.GetProjectByID(projectID)
	if err != nil {
		logger.Error().Err(err).Str("projectId", projectID.String()).Msg("Failed to get project")
		return nil, err
	}

	logs := make([]*log_model.Log, 0)

	for _, service := range project.Services {
		localLogs, err := l.GetLogsByServiceID(ctx, service.ID, timestamp, limit)
		if err != nil {
			logger.Error().Err(err).Str("serviceId", service.ID.String()).Msg("Failed to get logs")
			return nil, err
		}

		logs = append(logs, localLogs...)
	}

	return logs, nil
}

func (l *LogService) GetLogsByServiceID(ctx context.Context, serviceID uuid.UUID, timestamp time.Time, limit int) ([]*log_model.Log, error) {
	return nil, nil
}
