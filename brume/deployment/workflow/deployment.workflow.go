package deployment_workflow

import (
	"time"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/temporal/constants"
	"github.com/rs/zerolog/log"

	"go.temporal.io/sdk/workflow"
)

const UnhealthyCounter = 3
const ReadynessCheckInterval = time.Second * 3
const StatusCheckInterval = time.Second * 3

var logger = log.With().Str("module", "deployment_workflow").Logger()

type DeploymentWorkflow struct {
}

func NewDeploymentWorkflow() *DeploymentWorkflow {
	return &DeploymentWorkflow{}
}

// This is core.
// This workflow is used to deploy a version of a service, making
// sure the deployment is running, getting the logs and updating the service
// This run ONE service of the project. One deployment workflow must be run by service.
// This is responsible for the health of the service. Not logs and metrics. This is done
// at the machine scrapping level.
func (d *DeploymentWorkflow) DeploymentWorkflow(ctx workflow.Context, deployment *deployment_model.Deployment) error {
	logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Starting deployment workflow")

	shouldStop := false
	stopSignal := workflow.GetSignalChannel(ctx, temporal_constants.StopDeploymentSignal)

	// signal to stop the deployment
	// no idea how to inform the worker
	workflow.Go(ctx, func(ctx workflow.Context) {
		stopSignal.Receive(ctx, &shouldStop)
		logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Stop deployment signal received")
	})

	biddingWorkflowOpts := workflow.ChildWorkflowOptions{
		TaskQueue: temporal_constants.MasterTaskQueue,
	}

	ctx = workflow.WithChildOptions(ctx, biddingWorkflowOpts)

	// we start the bidding workflow
	// when the workflow is done thats mean the job has been accepted by an agent
	err := workflow.ExecuteChildWorkflow(ctx, temporal_constants.BidWorkflow, deployment).Get(ctx, nil)

	if err != nil {
		logger.Error().Err(err).Msg("Failed to start bidding workflow")
		return err
	}

	// monitoring of the deployment
	// this is where we do all the liveness logic, the scaling logic etc
	logger.Info().Str("deploymentId", deployment.ID.String()).Msg("Starting monitoring of the deployment")

	return nil
}
