package stack_activities

import (
	"context"

	"brume.dev/internal/log"
	stack_service "brume.dev/stack"
	stack_interfaces "brume.dev/stack/durable"
)

var logger = log.GetLogger("stack.activities")

type StackActivities struct {
	stackService *stack_service.StackService
}

func NewStackActivities(stackService *stack_service.StackService) *StackActivities {
	return &StackActivities{stackService: stackService}
}

func (a *StackActivities) UpdateStackStatus(ctx context.Context, input stack_interfaces.UpdateStackStatusInput) error {

	stack, err := a.stackService.GetStackByID(ctx, input.StackId)
	if err != nil {
		logger.Error().Err(err).Str("id", input.StackId).Msg("Failed to get stack by id")
		return err
	}

	stack.Status = input.Status

	_, err = a.stackService.UpdateStack(ctx, stack)

	if err != nil {
		logger.Error().Err(err).Str("id", input.StackId).Msg("Failed to update stack status")
		return err
	}

	return nil
}
