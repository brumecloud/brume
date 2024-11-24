package temporal_workflow

import (
	"brume.dev/container/docker"
	service_model "brume.dev/service/model"
	"github.com/rs/zerolog/log"

	"time"

	"go.temporal.io/sdk/workflow"
)

type DockerWorkflow struct {
	dockerActivity *docker.DockerActivity
}

func NewDockerWorkflow(dockerActivity *docker.DockerActivity) *DockerWorkflow {
	return &DockerWorkflow{dockerActivity: dockerActivity}
}

func (d *DockerWorkflow) RunServiceWorkflow(ctx workflow.Context, service *service_model.Service) error {
	opts := workflow.ActivityOptions{
		ScheduleToStartTimeout: time.Minute * 10,
		StartToCloseTimeout:    time.Minute * 10,
	}

	ctx = workflow.WithActivityOptions(ctx, opts)

	var containerId string
	err := workflow.ExecuteActivity(ctx, d.dockerActivity.StartService, service).Get(ctx, &containerId)

	if err != nil {
		return err
	}

	log.Info().Str("containerId", containerId).Msg("Service started, waiting for 30 seconds")

	workflow.Sleep(ctx, time.Second*30)

	err = workflow.ExecuteActivity(ctx, d.dockerActivity.StopService, service, containerId).Get(ctx, nil)

	if err != nil {
		return err
	}

	log.Info().Str("containerId", containerId).Msg("Service stopped")

	return nil
}
