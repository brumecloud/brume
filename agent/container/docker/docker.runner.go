package docker

import (
	"context"
	"encoding/binary"
	"fmt"
	"io"
	"strings"
	"time"

	deployment_model "brume.dev/deployment/model"
	log_model "brume.dev/logs/model"
	running_job "github.com/brumecloud/agent/job/model"
	"github.com/google/uuid"
)

type DockerEngineRunner struct {
	dockerService *DockerService
}

// this is outside of fx
func NewDockerEngineRunner(dockerService *DockerService) *DockerEngineRunner {
	logger.Info().Msg("Docker engine runner created")
	return &DockerEngineRunner{dockerService: dockerService}
}

func (d *DockerEngineRunner) StartJob(ctx context.Context, deployment *deployment_model.Deployment) (string, error) {
	logger.Info().Str("image", deployment.BuilderData.Image).Str("serviceId", deployment.ServiceID.String()).Msg("Starting container")

	image, err := d.dockerService.PullImage(deployment.BuilderData.Registry, deployment.BuilderData.Image, deployment.BuilderData.Tag)
	if err != nil {
		return "", err
	}

	containerId, err := d.dockerService.StartContainer(image, deployment.ServiceID, &deployment.RunnerData)
	if err != nil {
		return "", err
	}

	logger.Info().Str("containerId", containerId).Msg("Service started")

	return containerId, nil
}

func (d *DockerEngineRunner) StopJob(ctx context.Context, runningJob *running_job.RunningJob) error {
	container := runningJob.ContainerID
	if container == nil {
		logger.Error().Str("deploymentId", runningJob.ID).Msg("Trying to stop a non-running job")
		return nil
	}

	logger.Info().Str("containerId", *container).Msg("Stopping service")

	err := d.dockerService.StopContainer(*container)
	if err != nil {
		return err
	}

	logger.Debug().Str("containerId", *container).Msg("Removing container")
	err = d.dockerService.RemoveContainer(*container)

	return err
}

func (d *DockerEngineRunner) GetJobStatus(ctx context.Context, runningJob *running_job.RunningJob) (string, error) {
	if runningJob.ContainerID == nil {
		logger.Error().Str("deploymentId", runningJob.ID).Msg("Trying to get status of a non-running job")
		return "dead", nil
	}

	state, err := d.dockerService.StatusContainer(*runningJob.ContainerID)
	if err != nil {
		return "dead", err
	}

	return state.Status, nil
}

func (d *DockerEngineRunner) GetJobLogs(ctx context.Context, runningJob *running_job.RunningJob) ([]*log_model.Log, time.Time, error) {
	if runningJob.ContainerID == nil {
		logger.Error().Str("deploymentId", runningJob.ID).Msg("Container ID is empty")
		return nil, time.Now(), nil
	}

	now := time.Now()

	out, err := d.dockerService.GetLogs(*runningJob.ContainerID, runningJob.LastCheckAt)
	// need to convert the logs to the brume log format
	if err != nil {
		return nil, now, err
	}

	dockerLogsHeader := []byte{0, 0, 0, 0, 0, 0, 0, 0}
	// read the logs header
	n, err := out.Read(dockerLogsHeader)
	if err != nil {
		if err == io.EOF {
			logger.Debug().Str("containerId", runningJob.ID).Msg("No logs to return")
			return nil, now, nil
		}

		logger.Error().Err(err).Str("containerId", runningJob.ID).Msg("Error reading logs header")
		return nil, now, err
	}

	if n != 8 {
		logger.Error().Str("containerId", runningJob.ID).Msg("Invalid logs header")
		return nil, now, fmt.Errorf("invalid logs header")
	}

	var logType string

	if dockerLogsHeader[0] == 1 {
		logType = "stdout"
	} else {
		logType = "stderr"
	}

	count := binary.BigEndian.Uint32(dockerLogsHeader[4:8])

	if count == 0 {
		logger.Debug().Str("containerId", runningJob.ID).Msg("No logs to return")
		return nil, now, nil
	}

	data := make([]byte, count)

	n, err = out.Read(data)

	if n != int(count) {
		logger.Error().Str("containerId", runningJob.ID).Msg("Error reading logs content")
		return nil, now, fmt.Errorf("error reading logs content")
	}

	if err != nil {
		logger.Error().Str("containerId", runningJob.ID).Msg("Error reading logs content")
		return nil, now, err
	}

	logsLine := strings.Split(string(data), "\n")
	logs := make([]*log_model.Log, 0)

	for _, line := range logsLine {
		if line == "" {
			continue
		}

		fmt.Println("line", line)

		logs = append(logs, &log_model.Log{
			Message:      line,
			ServiceID:    uuid.MustParse(runningJob.ServiceID),
			DeploymentID: uuid.MustParse(runningJob.DeploymentID),
			Timestamp:    now,
			Level:        logType,
		})
	}

	return logs, now, nil
}

func (d *DockerEngineRunner) GetRunnerHealth(ctx context.Context) (string, error) {
	return "OK", nil
}
