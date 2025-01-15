package fx_job

import (
	job_service "github.com/brumecloud/agent/job"
	"go.uber.org/fx"
)

var JobModule = fx.Module("job",
	fx.Provide(job_service.NewJobService),
	fx.Invoke(func(jobService *job_service.JobService) {}),
)
