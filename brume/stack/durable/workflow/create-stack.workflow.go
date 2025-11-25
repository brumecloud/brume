package workflow_stack

import (
	"time"

	"brume.dev/internal/log"
	stack_interfaces "brume.dev/stack/durable"
	stack_model "brume.dev/stack/model"
	"go.temporal.io/sdk/temporal"
	"go.temporal.io/sdk/workflow"
)

var logger = log.GetLogger("stack.workflow")

func CreateStackWorkflow(ctx workflow.Context, input stack_interfaces.CreateStackWorkflowInput) error {
	logger.Info().Str("name", input.Name).Str("stack_id", input.Stack.ID).Str("cloud_account_id", input.CloudAccount.ID).Msg("Creating stack workflow")

	ao := workflow.ActivityOptions{
		StartToCloseTimeout: time.Second * 10,
		RetryPolicy: &temporal.RetryPolicy{
			MaximumAttempts:    3,
			MaximumInterval:    time.Second * 10,
			BackoffCoefficient: 2,
		},
	}
	ctx = workflow.WithActivityOptions(ctx, ao)

	// sleep for 10 seconds
	workflow.Sleep(ctx, time.Second*10)

	// update the stack status to deploying
	err := workflow.ExecuteActivity(ctx, stack_interfaces.UpdateStackStatusActivityName, stack_interfaces.UpdateStackStatusInput{
		StackId: input.Stack.ID,
		Status:  stack_model.StackStatusDeploying,
	}).Get(ctx, nil)
	if err != nil {
		logger.Error().Err(err).Str("stack_id", input.Stack.ID).Msg("Failed to update stack status to deploying")
		return err
	}

	workflow.Sleep(ctx, time.Second*20)

	err = workflow.ExecuteActivity(ctx, stack_interfaces.UpdateStackStatusActivityName, stack_interfaces.UpdateStackStatusInput{
		StackId: input.Stack.ID,
		Status:  stack_model.StackStatusDeployed,
	}).Get(ctx, nil)
	if err != nil {
		logger.Error().Err(err).Str("stack_id", input.Stack.ID).Msg("Failed to update stack status to deploying")
		return err
	}

	return nil
}
