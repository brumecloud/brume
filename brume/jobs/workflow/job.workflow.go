package job_workflows

import (
	"brume.dev/internal/log"
	job_service "brume.dev/jobs/service"
)

type JobWorkflow struct {
	jobService *job_service.JobService
}

var jobLogger = log.GetLogger("job_workflows").With().Str("workflow", "job").Logger()

func NewJobWorkflow(jobService *job_service.JobService) *JobWorkflow {
	return &JobWorkflow{
		jobService: jobService,
	}
}
