package docker

import (
	"context"
	"encoding/binary"
	"fmt"
	"io"
	"strings"
	"time"

	job_model "brume.dev/jobs/model"
	log_model "brume.dev/logs/model"
	runner_interfaces "github.com/brumecloud/agent/container/interfaces"
	running_job "github.com/brumecloud/agent/job/model"
)

type DockerEngineRunner struct {
	dockerService *DockerService
}

// this is outside of fx
func NewDockerEngineRunner(dockerService *DockerService) *DockerEngineRunner {
	logger.Info().Msg("Docker engine runner created")
	return &DockerEngineRunner{dockerService: dockerService}
}

func (d *DockerEngineRunner) StartJob(ctx context.Context, job *job_model.Job) (string, error) {
	logger.Info().Str("image", job.Deployment.BuilderData.Image).Str("serviceId", job.Deployment.ServiceID.String()).Msg("Starting container")

	image, err := d.dockerService.PullImage(job.Deployment.BuilderData.Registry, job.Deployment.BuilderData.Image, job.Deployment.BuilderData.Tag)
	if err != nil {
		return "", err
	}

	containerId, err := d.dockerService.StartContainer(image, job.ID, &job.Deployment.RunnerData)
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

// this will query the docker daemon and return the status of all the running jobs
// map[job-id] -> status
func (d *DockerEngineRunner) GetAllRunningJobs() (map[string]runner_interfaces.ContainerStatusResult, error) {
	containers, err := d.dockerService.GetAllRunningContainers()
	if err != nil {
		return nil, err
	}

	runningJobs := make(map[string]runner_interfaces.ContainerStatusResult)

	for _, container := range containers {
		runningJobs[container.Labels["brume.dev/job-id"]] = runner_interfaces.ContainerStatusResult{
			ContainerID: container.ID,
			JobID:       container.Labels["brume.dev/job-id"],
			Status:      dockerStateToJobStatus(container.State),
		}
	}

	return runningJobs, nil
}

func dockerStateToJobStatus(state string) job_model.JobStatusEnum {
	switch state {
	case "running":
		return job_model.JobStatusEnumRunning
	case "exited":
		return job_model.JobStatusEnumStopped
	default:
		return job_model.JobStatusEnumFailed
	}
}

func (d *DockerEngineRunner) GetJobStatus(ctx context.Context, runningJob *running_job.RunningJob) (job_model.JobStatusEnum, error) {
	if runningJob.ContainerID == nil {
		logger.Error().Str("deploymentId", runningJob.ID).Msg("Trying to get status of a non-running job")
		return job_model.JobStatusEnumFailed, nil
	}

	state, err := d.dockerService.StatusContainer(*runningJob.ContainerID)
	if err != nil {
		return job_model.JobStatusEnumFailed, err
	}

	switch state.Status {
	case "running":
		return job_model.JobStatusEnumRunning, nil
	case "exited":
		return job_model.JobStatusEnumStopped, nil
	default:
		return job_model.JobStatusEnumFailed, nil
	}
}

func (d *DockerEngineRunner) GetJobLogs(ctx context.Context, runningJob *running_job.RunningJob) ([]*log_model.AgentLogs, error) {
	if runningJob.ContainerID == nil {
		logger.Error().Str("deploymentId", runningJob.ID).Msg("Container ID is empty")
		return nil, nil
	}

	now := time.Now()

	out, err := d.dockerService.GetLogs(*runningJob.ContainerID, runningJob.LastCheckAt)
	// need to convert the logs to the brume log format
	if err != nil {
		return nil, err
	}

	dockerLogsHeader := []byte{0, 0, 0, 0, 0, 0, 0, 0}
	// read the logs header
	n, err := out.Read(dockerLogsHeader)
	if err != nil {
		if err == io.EOF {
			logger.Debug().Str("containerId", runningJob.ID).Msg("No logs to return")
			return nil, nil
		}

		logger.Error().Err(err).Str("containerId", runningJob.ID).Msg("Error reading logs header")
		return nil, err
	}

	if n != 8 {
		logger.Error().Str("containerId", runningJob.ID).Msg("Invalid logs header")
		return nil, fmt.Errorf("invalid logs header")
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
		return nil, nil
	}

	data := make([]byte, count)

	n, err = out.Read(data)

	if n != int(count) {
		logger.Error().Str("containerId", runningJob.ID).Msg("Error reading logs content")
		return nil, fmt.Errorf("error reading logs content")
	}

	if err != nil {
		logger.Error().Str("containerId", runningJob.ID).Msg("Error reading logs content")
		return nil, err
	}

	logsLine := strings.Split(string(data), "\n")
	logs := make([]*log_model.AgentLogs, 0)

	for _, line := range logsLine {
		if line == "" {
			continue
		}

		fmt.Println("line", line)

		logs = append(logs, &log_model.AgentLogs{
			JobID:     runningJob.ID,
			Message:   line,
			Level:     logType,
			Timestamp: now.Format(time.RFC3339),
		})
	}

	return logs, nil
}

func (d *DockerEngineRunner) GetRunnerHealth(ctx context.Context) (string, error) {
	return "OK", nil
}
