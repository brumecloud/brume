package project_workflow

import (
	"time"

	deployment_model "brume.dev/deployment/model"
	temporal_constants "brume.dev/internal/temporal/constants"
	"go.temporal.io/sdk/workflow"
)

type LogWorkflow struct {
}

func NewLogWorkflow() *LogWorkflow {
	return &LogWorkflow{}
}

func (l *LogWorkflow) RunLogWorkflow(ctx workflow.Context, deployment *deployment_model.Deployment) error {
	// logs polling will be done by brume cloud
	opts := workflow.ActivityOptions{
		ScheduleToStartTimeout: time.Minute * 10,
		StartToCloseTimeout:    time.Minute * 10,
		TaskQueue:              temporal_constants.MasterTaskQueue,
	}

	ctx = workflow.WithActivityOptions(ctx, opts)

	return nil
}
