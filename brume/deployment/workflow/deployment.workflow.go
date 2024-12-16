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
	log.Info().Str("deploymentId", deployment.ID.String()).Msg("Starting deployment workflow")

	opts := workflow.ActivityOptions{
		ScheduleToStartTimeout: time.Minute * 10,
		StartToCloseTimeout:    time.Minute * 10,
		TaskQueue:              temporal_constants.NodeTaskQueue,
	}

	shouldStop := false
	stopSignal := workflow.GetSignalChannel(ctx, temporal_constants.StopDeploymentSignal)

	workflow.Go(ctx, func(ctx workflow.Context) {
		stopSignal.Receive(ctx, &shouldStop)
		log.Info().Str("deploymentId", deployment.ID.String()).Msg("Stop deployment signal received")
	})

	ctx = workflow.WithActivityOptions(ctx, opts)

	// this part might need a rewrite because we only handle container runners for now.
	var containerId string

	// the queue should be something like "container:StartService"
	// and the queue the one associated with the tenant.
	// start the container, for now by scheduling it on the worker. Later with the betting system
	err := workflow.ExecuteActivity(ctx, temporal_constants.StartService, deployment).Get(ctx, &containerId)
	deployment.Execution.ContainerID = containerId

	if err != nil {
		log.Error().Err(err).Str("deploymentId", deployment.ID.String()).Msg("Failed to start the service")
		return err
	}

	// wait for the container to be ready
	workflow.Sleep(ctx, ReadynessCheckInterval)

	deployment.Execution.LastLogs = time.Now()
	unHealthyCounter := 0

	for {
		var result bool
		err := workflow.ExecuteActivity(ctx, temporal_constants.GetStatus, deployment).Get(ctx, &result)

		if err != nil {
			log.Error().Err(err).Str("deploymentId", deployment.ID.String()).Msg("Failed to get the service status")
		}

		// the container is not healthy, we need to restart it
		if !result {
			log.Debug().Str("deploymentId", deployment.ID.String()).Msg("Deployment is not healthy")

			unHealthyCounter++

			if unHealthyCounter > UnhealthyCounter {
				log.Debug().Str("deploymentId", deployment.ID.String()).Msg("Deployment is not alive")

				// start the same workflow.
				return workflow.NewContinueAsNewError(ctx, temporal_constants.DeploymentWorkflow, deployment)
			}
		}

		// sleep for the status check interval
		// or stop signal is received
		workflow.AwaitWithTimeout(ctx, StatusCheckInterval, func() bool {
			return shouldStop
		})

		if shouldStop {
			log.Debug().Str("deploymentId", deployment.ID.String()).Msg("Exiting the main health check loop")
			break
		}
	}

	// stop gracefully the container
	err = workflow.ExecuteActivity(ctx, temporal_constants.StopService, deployment).Get(ctx, nil)

	if err != nil {
		log.Error().Err(err).Str("deploymentId", deployment.ID.String()).Msg("Failed to stop the service")
	}

	return nil
}
