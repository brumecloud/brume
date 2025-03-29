package runner

import (
	"context"
	"errors"

	job_model "brume.dev/jobs/model"
	log_model "brume.dev/logs/model"
	runner_model "brume.dev/runner/model"
	"github.com/brumecloud/agent/container/docker"
	runner_interfaces "github.com/brumecloud/agent/container/interfaces"
	"github.com/brumecloud/agent/internal/log"
	running_job "github.com/brumecloud/agent/job/model"
)

var logger = log.GetLogger("runner")

type RunnerService struct {
	dockerRunner *docker.DockerEngineRunner
}

func NewRunnerService(dockerRunner *docker.DockerEngineRunner) *RunnerService {
	return &RunnerService{dockerRunner: dockerRunner}
}

// get the health of all the available runner types
func (r *RunnerService) GetRunnerHealth(ctx context.Context) (string, error) {
	return r.dockerRunner.GetRunnerHealth(ctx)
}

// TODO: also look for available runner type
func (r *RunnerService) StartJob(ctx context.Context, job *job_model.Job) (string, error) {
	logger.Info().Str("deploymentId", job.Deployment.ID.String()).Msg("Starting runner")

	if job.Deployment.RunnerData.Type == runner_model.RunnerTypeDocker {
		containerId, err := r.dockerRunner.StartJob(ctx, job)
		if err != nil {
			return "", err
		}
		return containerId, nil
	}

	return "", errors.New("unsupported runner type")
}

func (r *RunnerService) StopJob(ctx context.Context, runningJob *running_job.RunningJob) error {
	logger.Info().Str("deploymentId", runningJob.DeploymentID).Msg("Stopping runner")

	if runningJob.JobType == running_job.DockerRunningJob {
		r.dockerRunner.StopJob(ctx, runningJob)
		return nil
	}

	return errors.New("unsupported runner type, how did you get here?")
}

func (r *RunnerService) GetJobStatus(ctx context.Context, runningJob *running_job.RunningJob) (job_model.JobStatusEnum, error) {
	if runningJob.JobType == running_job.DockerRunningJob {
		return r.dockerRunner.GetJobStatus(ctx, runningJob)
	}

	return "", errors.New("unsupported job type, how did you get here?")
}

func (r *RunnerService) GetLogs(ctx context.Context, runningJob *running_job.RunningJob) ([]*log_model.AgentLogs, error) {
	if runningJob.JobType == running_job.DockerRunningJob {
		return r.dockerRunner.GetJobLogs(ctx, runningJob)
	}

	return nil, errors.New("unsupported runner type, how did you get here?")
}

// todo: implement this for all the runners
func (r *RunnerService) GetAllRunningJobs() (map[string]runner_interfaces.ContainerStatusResult, error) {
	jobs, err := r.dockerRunner.GetAllRunningJobs()
	if err != nil {
		return nil, err
	}

	logger.Trace().Interface("jobs", jobs).Msg("Got all running jobs (from docker engine)")

	return jobs, nil
}
