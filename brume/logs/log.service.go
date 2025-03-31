package log

import (
	"context"
	"fmt"
	"strings"

	deployment_service "brume.dev/deployment"
	deployment_model "brume.dev/deployment/model"
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
func (l *LogService) GetLogs(ctx context.Context, projectID uuid.UUID) ([]*log_model.Log, error) {
	logger.Trace().Str("projectId", projectID.String()).Msg("Getting logs")

	project, err := l.ProjectService.GetProjectByID(projectID)
	if err != nil {
		logger.Error().Err(err).Str("projectId", projectID.String()).Msg("Failed to get project")
		return nil, err
	}

	logs := make([]*log_model.Log, 0)

	for _, service := range project.Services {
		localLogs, err := l.GetLogsByServiceID(ctx, service.ID)
		if err != nil {
			logger.Error().Err(err).Str("serviceId", service.ID.String()).Msg("Failed to get logs")
			return nil, err
		}

		logs = append(logs, localLogs...)
	}

	return logs, nil
}

func (l *LogService) GetLogsByServiceID(ctx context.Context, serviceID uuid.UUID) ([]*log_model.Log, error) {
	containersIds := make([]string, 0)

	// get all the jobs for the service
	jobs, err := l.jobService.GetJobsByServiceID(serviceID)
	if err != nil {
		logger.Error().Err(err).Str("serviceId", serviceID.String()).Msg("Failed to get jobs")
		return nil, err
	}

	containerDeploymentMap := make(map[string]*deployment_model.Deployment)

	for _, job := range jobs {
		if job.ContainerID != nil {
			containersIds = append(containersIds, fmt.Sprintf("'%s'", *job.ContainerID))

			deployment, err := l.DeploymentService.GetDeployment(*job.DeploymentID)
			if err != nil {
				logger.Warn().Err(err).Str("deploymentId", job.DeploymentID.String()).Msg("Failed to get deployment")
				continue
			} else {
				containerDeploymentMap[*job.ContainerID] = deployment
			}
		}
	}

	logger.Info().Interface("containersIds", containersIds).Msg("Containers ids")

	if len(containersIds) == 0 {
		return nil, nil
	}

	logs := make([]*log_model.Log, 0)

	query := fmt.Sprintf("SELECT `Timestamp`, `SeverityText`, `LogAttributes` FROM brume.otel_logs WHERE LogAttributes['container_id'] IN (%s)", strings.Join(containersIds, ","))

	logger.Debug().Str("query", query).Msg("Query")

	rows, err := l.chdb.Conn.Query(context.Background(), query)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to get logs")
		return nil, err
	}

	for rows.Next() {
		var raw log_model.RawLog
		err = rows.Scan(&raw.Timestamp, &raw.SeverityText, &raw.LogAttributes)

		if err != nil {
			logger.Error().Err(err).Msg("Failed to scan log")
		}

		deployment := containerDeploymentMap[raw.LogAttributes["container_id"]]

		// how??
		if deployment == nil {
			logger.Error().Str("containerId", raw.LogAttributes["container_id"]).Msg("Deployment not found")
			continue
		} else {
			log := log_model.Log{
				ContainerID:    raw.LogAttributes["container_id"],
				Message:        raw.LogAttributes["body"],
				Level:          raw.LogAttributes["level"],
				Timestamp:      raw.Timestamp,
				ServiceID:      serviceID.String(),
				DeploymentID:   deployment.ID.String(),
				DeploymentName: deployment.Name,
			}

			logs = append(logs, &log)
		}
	}

	return logs, nil
}
