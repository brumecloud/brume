package runner

import (
	"context"
	"errors"
	"time"

	deployment_model "brume.dev/deployment/model"
	log_model "brume.dev/logs/model"
	runner_model "brume.dev/runner/model"
	"github.com/brumecloud/agent/container/docker"
	running_job "github.com/brumecloud/agent/job/model"
	"github.com/rs/zerolog/log"
)

var logger = log.With().Str("module", "runner").Logger()

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
func (r *RunnerService) StartJob(ctx context.Context, deployment *deployment_model.Deployment) (string, error) {
	logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Starting runner")

	if deployment.RunnerData.Type == runner_model.RunnerTypeDocker {
		containerId, err := r.dockerRunner.StartJob(ctx, deployment)
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

func (r *RunnerService) GetJobStatus(ctx context.Context, runningJob *running_job.RunningJob) (string, error) {
	if runningJob.JobType == running_job.DockerRunningJob {
		return r.dockerRunner.GetJobStatus(ctx, runningJob)
	}

	return "", errors.New("unsupported job type, how did you get here?")
}

func (r *RunnerService) GetLogs(ctx context.Context, runningJob *running_job.RunningJob) ([]*log_model.Log, time.Time, error) {
	if runningJob.JobType == running_job.DockerRunningJob {
		return r.dockerRunner.GetJobLogs(ctx, runningJob)
	}

	return nil, time.Time{}, errors.New("unsupported runner type, how did you get here?")
}
