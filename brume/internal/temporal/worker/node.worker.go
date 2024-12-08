package temporal_worker

import (
	"brume.dev/container"
	"brume.dev/internal/temporal/constants"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/activity"
	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/worker"
)

type NodeWorker struct {
}

func StartNodeWorker(temporalClient client.Client, containerActivity *container.ContainerActivity) *NodeWorker {
	w := worker.New(temporalClient, temporal_constants.NodeTaskQueue, worker.Options{})

	w.RegisterActivityWithOptions(containerActivity.StartService, activity.RegisterOptions{Name: temporal_constants.StartService})
	w.RegisterActivityWithOptions(containerActivity.StopService, activity.RegisterOptions{Name: temporal_constants.StopService})
	w.RegisterActivityWithOptions(containerActivity.GetLogs, activity.RegisterOptions{Name: temporal_constants.GetLogs})

	log.Warn().Msg("Starting temporal node worker")

	err := w.Run(worker.InterruptCh())

	if err != nil {
		log.Error().Err(err).Msg("Error starting node worker")
		panic(err)
	}

	return &NodeWorker{}
}
