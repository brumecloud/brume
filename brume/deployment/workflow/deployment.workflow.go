package deployment_workflow

import (
	"time"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/log"
	temporal_constants "brume.dev/internal/temporal/constants"
	job_model "brume.dev/jobs/model"
	job_service "brume.dev/jobs/service"

	"go.temporal.io/sdk/workflow"
)

const (
	UnhealthyCounter       = 3
	ReadynessCheckInterval = time.Second * 3
	StatusCheckInterval    = time.Second * 3
)

var logger = log.GetLogger("deployment_workflow")

type DeploymentWorkflow struct {
	jobService *job_service.JobService
}

func NewDeploymentWorkflow(jobService *job_service.JobService) *DeploymentWorkflow {
	return &DeploymentWorkflow{jobService: jobService}
}

// This workflow is used to deploy a version of a service, making
// sure the deployment is running, getting the logs and updating the service
// This run ONE service of the project. One deployment workflow must be run by service.
// This is responsible for the health of the service. Not logs and metrics. This is done
// at the machine scrapping level.
func (d *DeploymentWorkflow) DeploymentWorkflow(ctx workflow.Context, deployment *deployment_model.Deployment) error {
	logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Starting deployment workflow")
	workflowID := workflow.GetInfo(ctx).WorkflowExecution.ID
	runID := workflow.GetInfo(ctx).WorkflowExecution.RunID

	unhealthyCounter := 0
	err := workflow.SetUpdateHandlerWithOptions(ctx, temporal_constants.UnhealthyJobSignal, func(ctx workflow.Context, data interface{}) error {
		unhealthyCounter++
		return nil
	}, workflow.UpdateHandlerOptions{})
	if err != nil {
		logger.Error().Err(err).Msg("Failed to set unhealthy job signal handler")
		return err
	}

	shouldStop := false
	err = workflow.SetUpdateHandlerWithOptions(ctx, temporal_constants.StopJobSignal, func(ctx workflow.Context, data interface{}) error {
		logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Stop deployment signal received")
		shouldStop = true
		return nil
	}, workflow.UpdateHandlerOptions{})
	if err != nil {
		logger.Error().Err(err).Msg("Failed to set stop job signal handler")
		return err
	}

	// create a job for the deployment
	// the job is linked to the deployment which created it
	// because this is where the unhealthy counter is set
	job, err := d.jobService.CreateJob(deployment, workflowID, runID)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to create job")
		return err
	}

	err = d.startBidding(ctx, job)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to start bidding workflow")
		return err
	}

	// we wait for an update on the job : the job is unhealthy or the deployment is stopped
	for {
		// we wait for the unhealthy counter to be greater than 3 or the stop signal to be received
		workflow.Await(ctx, func() bool {
			return unhealthyCounter >= UnhealthyCounter || shouldStop
		})

		if unhealthyCounter >= UnhealthyCounter {
			logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Unhealthy counter is greater than 3, starting bidding workflow")
			newJob, err := d.jobService.CreateJob(deployment, workflowID, runID)
			if err != nil {
				logger.Error().Err(err).Msg("Failed to create job")
				return err
			}

			err = d.startBidding(ctx, newJob)
			if err != nil {
				logger.Error().Err(err).Msg("Failed to start bidding workflow of the new job")
				return err
			}

			err = d.jobService.SetJobStatus(job.ID, job_model.JobStatusEnumStopped)
			if err != nil {
				logger.Error().Err(err).Msg("Failed to set job status of the old job")
				return err
			}

			job = newJob
			newJob = nil

			unhealthyCounter = 0
			shouldStop = false
		}

		if shouldStop {
			logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Stop deployment signal received")
			return nil
		}
	}
}

func (d *DeploymentWorkflow) startBidding(ctx workflow.Context, job *job_model.Job) error {
	biddingWorkflowOpts := workflow.ChildWorkflowOptions{
		TaskQueue: temporal_constants.MasterTaskQueue,
	}

	ctx = workflow.WithChildOptions(ctx, biddingWorkflowOpts)

	// we start the bidding workflow
	// when the workflow is done thats mean the job has been accepted by an agent
	err := workflow.ExecuteChildWorkflow(ctx, temporal_constants.BidWorkflow, job).Get(ctx, nil)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to start bidding workflow")
		return err
	}

	return nil
}
