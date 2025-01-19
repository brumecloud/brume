package runner

import (
	"context"
	"errors"
	"time"

	deployment_model "brume.dev/deployment/model"
	log_model "brume.dev/logs/model"
	runner_model "brume.dev/runner/model"
	"github.com/brumecloud/agent/container/docker"
	"github.com/rs/zerolog/log"
)

var logger = log.With().Str("module", "runner").Logger()

type RunnerService struct {
	dockerRunner *docker.DockerEngineRunner
}

func NewRunnerService(dockerRunner *docker.DockerEngineRunner) *RunnerService {
	return &RunnerService{dockerRunner: dockerRunner}
}

// TODO: also look for available runner type
func (r *RunnerService) StartJob(ctx context.Context, deployment *deployment_model.Deployment) error {
	logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Starting runner")

	if deployment.RunnerData.Type == runner_model.RunnerTypeDocker {
		r.dockerRunner.StartJob(ctx, deployment)
		return nil
	}

	return errors.New("unsupported runner type")
}

func (r *RunnerService) StopJob(ctx context.Context, deployment *deployment_model.Deployment) error {
	logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Stopping runner")

	if deployment.RunnerData.Type == runner_model.RunnerTypeDocker {
		r.dockerRunner.StopJob(ctx, deployment)
		return nil
	}

	return errors.New("unsupported runner type, how did you get here?")
}

func (r *RunnerService) GetJobStatus(ctx context.Context, deployment *deployment_model.Deployment) (string, error) {
	if deployment.RunnerData.Type == runner_model.RunnerTypeDocker {
		return r.dockerRunner.GetJobStatus(ctx, deployment)
	}

	return "", errors.New("unsupported runner type, how did you get here?")
}

func (r *RunnerService) GetLogs(ctx context.Context, deployment *deployment_model.Deployment) ([]*log_model.Log, time.Time, error) {
	if deployment.RunnerData.Type == runner_model.RunnerTypeDocker {
		return r.dockerRunner.GetJobLogs(ctx, deployment)
	}

	return nil, time.Time{}, errors.New("unsupported runner type, how did you get here?")
}
