package job_workflows

import (
	"brume.dev/internal/log"
	job_model "brume.dev/jobs/model"
	job_service "brume.dev/jobs/service"
	"go.temporal.io/sdk/workflow"
)

type BiddingWorkflow struct {
	bidService *job_service.BidService
	jobService *job_service.JobService
}

var bidLogger = log.GetLogger("job_workflows").With().Str("workflow", "bid").Logger()

func NewBiddingWorkflow(bidService *job_service.BidService, jobService *job_service.JobService) *BiddingWorkflow {
	return &BiddingWorkflow{
		bidService: bidService,
		jobService: jobService,
	}
}

// this workflow is responsible for the bidding process
// of the job, for the moment, we accept any bid (first come first serve)
// in the future, agent will be evaluated and the best will be selected
// this bidding logic will have a lot of businnes constraints inside (network topology, machine type, etc)
func (b *BiddingWorkflow) BidWorkflow(ctx workflow.Context, job *job_model.Job) error {
	bidLogger.Info().Interface("deployment", job.Deployment).Msg("Starting bid workflow")
	workflowID := workflow.GetInfo(ctx).WorkflowExecution.ID
	runID := workflow.GetInfo(ctx).WorkflowExecution.RunID

	job.BidWorkflowID = &workflowID
	job.BidRunID = &runID
	job.Price = 1000
	job.Status = job_model.JobStatusEnumPending

	err := b.bidService.UpdateBid(job)
	if err != nil {
		bidLogger.Error().Err(err).Msg("Failed to update bid")
		return err
	}

	// brume waits for an agent to make a bid
	// then its attribute the bid to the highest bidder
	machineFound := false

	// this will be updated by the bidding service
	err = workflow.SetUpdateHandler(ctx, "machine_found", func(ctx workflow.Context, signalName string, _ ...any) error {
		machineFound = true
		return nil
	})
	if err != nil {
		bidLogger.Error().Err(err).Msg("Failed to set update handler")
		return err
	}

	// we wait for the machine to be found
	// this is where will we do the bidding logic
	// update by the digging service
	workflow.Await(ctx, func() bool {
		return machineFound
	})

	// update the job status to running when the machine is found
	// when a machine is found, the job is considered running
	b.jobService.SetJobStatus(job.ID, job_model.JobStatusEnumRunning)

	bidLogger.Info().Msg("Machine found, bidding process finished")

	return nil
}
