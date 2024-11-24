package temporal_workflow

import (
	"brume.dev/container/docker"
	service_model "brume.dev/service/model"

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

	err := workflow.ExecuteActivity(ctx, d.dockerActivity.StartContainer, service).Get(ctx, nil)

	return err
}
