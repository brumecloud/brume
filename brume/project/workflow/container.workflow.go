package project_workflow

import (
	"time"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/log"
	temporal_constants "brume.dev/internal/temporal/constants"
	"go.temporal.io/sdk/workflow"
)

var logger = log.GetLogger("container_workflow")

type ContainerWorkflow struct{}

func NewContainerWorkflow() *ContainerWorkflow {
	return &ContainerWorkflow{}
}

func (d *ContainerWorkflow) RunContainerDeploymentWorkflow(ctx workflow.Context, deployment *deployment_model.Deployment) error {
	opts := workflow.ActivityOptions{
		ScheduleToStartTimeout: time.Minute * 10,
		StartToCloseTimeout:    time.Minute * 10,
		TaskQueue:              temporal_constants.NodeTaskQueue,
	}

	ctx = workflow.WithActivityOptions(ctx, opts)

	var containerId string
	err := workflow.ExecuteActivity(ctx, temporal_constants.StartService, deployment).Get(ctx, &containerId)
	if err != nil {
		return err
	}

	logger.Info().Str("containerId", containerId).Msg("Service started, waiting for 30 seconds")

	// this is not saved in db yet.
	// we can do that because temporal saves the state of the workflow
	deployment.Execution.ContainerID = containerId
	deployment.Execution.LastLogs = time.Now()

	return nil
}
