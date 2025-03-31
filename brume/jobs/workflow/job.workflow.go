package job_workflows

import (
	job_service "brume.dev/jobs/service"
)

type JobWorkflow struct {
	jobService *job_service.JobService
}

func NewJobWorkflow(jobService *job_service.JobService) *JobWorkflow {
	return &JobWorkflow{
		jobService: jobService,
	}
}
