package job_fx

import (
	job_service "brume.dev/jobs/service"
	bid_workflow "brume.dev/jobs/workflow"
	"go.uber.org/fx"
)

var JobModule = fx.Module("job",
	fx.Provide(job_service.NewBidService),
	fx.Provide(bid_workflow.NewBiddingWorkflow),
	fx.Provide(job_service.NewJobService),
	fx.Invoke(func(bidService *job_service.BidService, biddingWorkflow *bid_workflow.BiddingWorkflow) {}),
)
