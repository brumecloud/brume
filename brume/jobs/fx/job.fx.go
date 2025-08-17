package job_fx

import (
	job_service "brume.dev/jobs/service"
	"go.uber.org/fx"
)

var JobModule = fx.Module("job",
	fx.Provide(job_service.NewBidService),
	fx.Provide(job_service.NewJobService),
	fx.Invoke(func(bidService *job_service.BidService) {}),
)
