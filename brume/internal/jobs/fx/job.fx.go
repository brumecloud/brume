package job_fx

import (
	"brume.dev/internal/jobs/service"
	"go.uber.org/fx"
)

var JobModule = fx.Module("job",
	fx.Provide(job_service.NewBidService),
	fx.Invoke(func(bidService *job_service.BidService) {}),
)
