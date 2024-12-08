package temporal_worker

import (
	"brume.dev/internal/temporal/constants"
	temporal_workflow "brume.dev/internal/temporal/workflow"
	brume_log "brume.dev/logs"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/activity"
	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/worker"
	"go.temporal.io/sdk/workflow"
)

type MasterWorker struct {
}

func StartMasterWorker(temporalClient client.Client, logActivity *brume_log.LogActivity, containerWorkflow *temporal_workflow.ContainerWorkflow) *MasterWorker {
	w := worker.New(temporalClient, temporal_constants.MasterTaskQueue, worker.Options{})

	w.RegisterActivityWithOptions(logActivity.IngestLogs, activity.RegisterOptions{Name: temporal_constants.IngestLogs})
	w.RegisterWorkflowWithOptions(containerWorkflow.RunContainerDeploymentWorkflow, workflow.RegisterOptions{Name: temporal_constants.RunContainerDeploymentWorkflow})

	log.Warn().Msg("Starting temporal master worker")

	err := w.Run(worker.InterruptCh())

	if err != nil {
		log.Error().Err(err).Msg("Error starting master worker")
		panic(err)
	}

	return &MasterWorker{}
}
