package temporal_worker

import (
	deployment_workflow "brume.dev/deployment/workflow"
	bid_workflow "brume.dev/internal/jobs/workflow"
	temporal_constants "brume.dev/internal/temporal/constants"
	brume_log "brume.dev/logs"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/activity"
	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/worker"
	"go.temporal.io/sdk/workflow"
)

type MasterWorker struct {
}

func StartMasterWorker(temporalClient client.Client, logActivity *brume_log.LogActivity, deploymentWorkflow *deployment_workflow.DeploymentWorkflow, biddingWorkflow *bid_workflow.BiddingWorkflow) *MasterWorker {
	w := worker.New(temporalClient, temporal_constants.MasterTaskQueue, worker.Options{})

	w.RegisterActivityWithOptions(logActivity.IngestLogs, activity.RegisterOptions{Name: temporal_constants.IngestLogs})
	w.RegisterWorkflowWithOptions(deploymentWorkflow.DeploymentWorkflow, workflow.RegisterOptions{Name: temporal_constants.DeploymentWorkflow})
	w.RegisterWorkflowWithOptions(biddingWorkflow.BidWorkflow, workflow.RegisterOptions{Name: temporal_constants.BidWorkflow})

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
