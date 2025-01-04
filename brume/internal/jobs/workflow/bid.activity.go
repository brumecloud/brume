package bid_workflow

import (
	deployment_model "brume.dev/deployment/model"
	job_service "brume.dev/internal/jobs/service"
	brume_log "brume.dev/internal/log"
	"go.temporal.io/sdk/workflow"
)

type BiddingWorkflow struct {
	bidService *job_service.BidService
}

var logger = brume_log.GetLogger("bid_workflow")

func NewBiddingWorkflow(bidService *job_service.BidService) *BiddingWorkflow {
	return &BiddingWorkflow{
		bidService: bidService,
	}
}

func (b *BiddingWorkflow) BidWorkflow(ctx workflow.Context, deployment *deployment_model.Deployment) error {
	logger.Info().Interface("deployment", deployment).Msg("Starting bid workflow")
	workflowID := workflow.GetInfo(ctx).WorkflowExecution.ID
	runID := workflow.GetInfo(ctx).WorkflowExecution.RunID

	// we create the bid, any agent can pick it
	bid, err := b.bidService.CreateBid(deployment, deployment.ServiceID, workflowID, runID)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to create bid")
		return err
	}

	logger.Info().Interface("bid", bid).Msg("Bid created")

	// brume waits for an agent to make a bid
	// then its attribute the bid to the highest bidder
	machineFound := false

	err = workflow.SetUpdateHandler(ctx, "machine_found", func(ctx workflow.Context, signalName string, _ ...any) error {
		machineFound = true
		return nil
	})
	if err != nil {
		logger.Error().Err(err).Msg("Failed to set update handler")
		return err
	}

	// we wait for the machine to be found
	// this is where will we do the bidding logic
	workflow.Await(ctx, func() bool {
		return machineFound
	})

	logger.Info().Msg("Machine found, bidding process finished")

	return nil
}
