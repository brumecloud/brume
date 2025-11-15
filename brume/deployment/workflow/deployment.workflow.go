package deployment_workflow

import (
	"context"
	"errors"
	"time"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/config"
	"brume.dev/internal/log"
	job_model "brume.dev/jobs/model"
	job_service "brume.dev/jobs/service"
)

const (
	JobFailedCounter       = 5
	ReadynessCheckInterval = time.Second * 3
	StatusCheckInterval    = time.Second * 3
)

var logger = log.GetLogger("deployment.workflow")

type DeploymentWorkflow struct {
	jobService *job_service.JobService
	cfg        *config.BrumeConfig
}

func NewDeploymentWorkflow(jobService *job_service.JobService, cfg *config.BrumeConfig) *DeploymentWorkflow {
	return &DeploymentWorkflow{jobService: jobService, cfg: cfg}
}

// This workflow is used to deploy a version of a service, making
// sure the deployment is running, getting the logs and updating the service
// This run ONE service of the project. One deployment workflow must be run by service.
// This is responsible for the health of the service. Not logs and metrics. This is done
// at the machine scrapping level.
func (d *DeploymentWorkflow) DeploymentWorkflow(ctx context.Context, deployment *deployment_model.Deployment) error {
	logger.Trace().Str("deployment_id", deployment.ID.String()).Msg("Starting deployment workflow")
	return errors.New("not implemented")
}

func (d *DeploymentWorkflow) startBidding(ctx context.Context, job *job_model.Job) error {
	return errors.New("not implemented")
}
