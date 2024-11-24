package temporal_worker

import (
	"brume.dev/container/docker"
	temporal_workflow "brume.dev/internal/temporal/workflow"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/activity"
	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/worker"
	"go.temporal.io/sdk/workflow"
)

type NodeWorker struct {
}

func StartNodeWorker(temporalClient client.Client, dockerWorkflow *temporal_workflow.DockerWorkflow, dockerActivity *docker.DockerActivity) *NodeWorker {
	w := worker.New(temporalClient, "node", worker.Options{})

	w.RegisterWorkflowWithOptions(dockerWorkflow.RunServiceWorkflow, workflow.RegisterOptions{Name: "RunServiceWorkflow"})
	w.RegisterActivityWithOptions(dockerActivity.StartContainer, activity.RegisterOptions{Name: "StartContainer"})

	log.Info().Msg("Starting node worker")

	err := w.Run(worker.InterruptCh())

	if err != nil {
		log.Error().Err(err).Msg("Error starting node worker")
		panic(err)
	}

	return &NodeWorker{}
}
