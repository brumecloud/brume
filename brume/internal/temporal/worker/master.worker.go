package temporal_worker

import (
	temporal_constants "brume.dev/internal/temporal/constants"
	brume_log "brume.dev/logs"
	container_workflow "brume.dev/project/workflow"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/activity"
	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/worker"
	"go.temporal.io/sdk/workflow"
)

type MasterWorker struct {
}

func StartMasterWorker(temporalClient client.Client, logActivity *brume_log.LogActivity, containerWorkflow *container_workflow.ContainerWorkflow) *MasterWorker {
	w := worker.New(temporalClient, temporal_constants.MasterTaskQueue, worker.Options{})

	w.RegisterActivityWithOptions(logActivity.IngestLogs, activity.RegisterOptions{Name: temporal_constants.IngestLogs})
	w.RegisterWorkflowWithOptions(containerWorkflow.RunContainerDeploymentWorkflow, workflow.RegisterOptions{Name: temporal_constants.RunContainerDeploymentWorkflow})

	log.Warn().Msg("Starting temporal master worker")

	go func() {
		err := w.Run(worker.InterruptCh())

		if err != nil {
			log.Error().Err(err).Msg("Error starting master worker")
			panic(err)
		}
	}()

	return &MasterWorker{}
}
