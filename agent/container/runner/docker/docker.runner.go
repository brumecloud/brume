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
	runner_interfaces "brume.dev/runner/interfaces"
	"github.com/rs/zerolog/log"
)

type DockerEngineRunner struct {
	runner_interfaces.ContainerRunner
	dockerService *DockerService
}

// this is outside of fx
func NewDockerEngineRunner(dockerService *DockerService) *DockerEngineRunner {
	log.Info().Msg("Docker engine runner created")
	return &DockerEngineRunner{dockerService: dockerService}
}

func (d *DockerEngineRunner) StartService(ctx context.Context, deployment *deployment_model.Deployment) (string, error) {
	log.Info().Str("image", deployment.BuilderData.Image).Str("serviceId", deployment.ServiceID.String()).Msg("Starting container")

	image, err := d.dockerService.PullImage(deployment.BuilderData.Registry, deployment.BuilderData.Image, deployment.BuilderData.Tag)

	if err != nil {
		return "", err
	}

	containerId, err := d.dockerService.StartContainer(image, deployment.ServiceID, &deployment.RunnerData)

	if err != nil {
		return "", err
	}

	log.Info().Str("containerId", containerId).Msg("Service started")

	return containerId, nil
}

func (d *DockerEngineRunner) StopService(ctx context.Context, deployment *deployment_model.Deployment) error {
	log.Info().Str("containerId", deployment.Execution.ContainerID).Str("service", deployment.ServiceID.String()).Msg("Stopping service")

	err := d.dockerService.StopContainer(deployment.Execution.ContainerID)

	if err != nil {
		return err
	}

	log.Debug().Str("containerId", deployment.Execution.ContainerID).Msg("Removing container")
	err = d.dockerService.RemoveContainer(deployment.Execution.ContainerID)

	return err
}

func (d *DockerEngineRunner) GetStatus(ctx context.Context, deployment *deployment_model.Deployment) (bool, error) {
	state, err := d.dockerService.StatusContainer(deployment.Execution.ContainerID)

	if err != nil {
		return false, err
	}

	return state.Running, nil
}

func (d *DockerEngineRunner) GetLogs(ctx context.Context, deployment *deployment_model.Deployment) ([]*log_model.Log, time.Time, error) {
	if deployment.Execution.ContainerID == "" {
		log.Error().Str("deploymentId", deployment.ID.String()).Msg("Container ID is empty")
		return nil, time.Now(), nil
	}

	now := time.Now()

	out, err := d.dockerService.GetLogs(deployment.Execution.ContainerID, deployment.Execution.LastLogs)

	// need to convert the logs to the brume log format

	if err != nil {
		return nil, now, err
	}

	dockerLogsHeader := []byte{0, 0, 0, 0, 0, 0, 0, 0}
	// read the logs header
	n, err := out.Read(dockerLogsHeader)

	if err != nil {
		if err == io.EOF {
			log.Debug().Str("containerId", deployment.Execution.ContainerID).Msg("No logs to return")
			return nil, now, nil
		}

		log.Error().Err(err).Str("containerId", deployment.Execution.ContainerID).Msg("Error reading logs header")
		return nil, now, err
	}

	if n != 8 {
		log.Error().Str("containerId", deployment.Execution.ContainerID).Msg("Invalid logs header")
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
		log.Debug().Str("containerId", deployment.Execution.ContainerID).Msg("No logs to return")
		return nil, now, nil
	}

	data := make([]byte, count)

	n, err = out.Read(data)

	if n != int(count) {
		log.Error().Str("containerId", deployment.Execution.ContainerID).Msg("Error reading logs content")
		return nil, now, fmt.Errorf("error reading logs content")
	}

	if err != nil {
		log.Error().Str("containerId", deployment.Execution.ContainerID).Msg("Error reading logs content")
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
			Message:        line,
			ServiceID:      deployment.ServiceID,
			DeploymentID:   deployment.ID,
			DeploymentName: fmt.Sprintf("%s-%.6s-%s", deployment.ServiceName, deployment.ID.String(), deployment.Env),
			ProjectID:      deployment.ProjectID,
			Timestamp:      now,
			Level:          logType,
		})
	}

	return logs, now, nil
}
