package stack_fx

import (
	stack_service "brume.dev/stack"
	stack_activities "brume.dev/stack/durable/activities"
	"go.uber.org/fx"
)

var StackModule = fx.Module("stack",
	fx.Provide(stack_service.NewStackService, stack_activities.NewStackActivities),
	fx.Invoke(func(s *stack_service.StackService) {}),
)
