package fx_temporal

import (
	"brume.dev/internal/temporal"
	"brume.dev/internal/temporal/worker"
	"brume.dev/internal/temporal/workflow"
	"go.temporal.io/sdk/client"
	"go.uber.org/fx"
)

var TemporalModule = fx.Module("temporal",
	fx.Provide(temporal.NewClient),
	fx.Invoke(func(c client.Client) {}),
)

var ContainerWorkflowModule = fx.Module("container-workflow",
	fx.Provide(temporal_workflow.NewContainerWorkflow),
	fx.Invoke(func(w *temporal_workflow.ContainerWorkflow) {}),
)

var TemporalNodeModule = fx.Module("temporal-node",
	fx.Provide(temporal_worker.StartNodeWorker),
	fx.Invoke(func(c client.Client) {}),
)

var TemporalMasterModule = fx.Module("temporal-master",
	fx.Provide(temporal_worker.StartMasterWorker),
	fx.Invoke(func(c client.Client) {}),
)
