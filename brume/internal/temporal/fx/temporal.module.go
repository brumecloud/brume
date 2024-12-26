package fx_temporal

import (
	"brume.dev/internal/temporal"
	"brume.dev/internal/temporal/worker"
	"go.temporal.io/sdk/client"
	"go.uber.org/fx"
)

var TemporalModule = fx.Module("temporal",
	fx.Provide(temporal.NewClient),
	fx.Invoke(func(c client.Client) {}),
)

var TemporalOrchestratorModule = fx.Module("temporal-orchestrator",
	fx.Provide(temporal_worker.StartMasterWorker),
)
