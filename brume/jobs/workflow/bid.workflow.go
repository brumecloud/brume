package job_workflows

import (
	"context"
	"errors"

	"brume.dev/internal/log"
	job_model "brume.dev/jobs/model"
	job_service "brume.dev/jobs/service"
)

type BiddingWorkflow struct {
	bidService *job_service.BidService
	jobService *job_service.JobService
}

var bidLogger = log.GetLogger("job_workflow").With().Str("workflow", "bid").Logger()

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
func (b *BiddingWorkflow) BidWorkflow(ctx context.Context, job *job_model.Job) error {
	bidLogger.Info().Str("job_id", job.ID.String()).Msg("Starting bidding workflow")
	return errors.New("not implemented")
}
