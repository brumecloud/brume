package docker

import (
	"context"

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
		logger.Error().Err(err).Msg("Failed to get all running containers")
		return nil, err
	}

	logger.Info().Int("containers", len(containers)).Msg("Got all running containers")

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
	return []*log_model.AgentLogs{}, nil
}

func (d *DockerEngineRunner) GetRunnerHealth(ctx context.Context) (string, error) {
	return "OK", nil
}
