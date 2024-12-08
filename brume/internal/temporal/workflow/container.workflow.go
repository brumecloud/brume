package temporal_workflow

import (
	"brume.dev/internal/temporal/constants"
	service_model "brume.dev/service/model"
	"github.com/rs/zerolog/log"

	brume_logs "brume.dev/logs/model"
	"time"

	"go.temporal.io/sdk/workflow"
)

type ContainerWorkflow struct {
}

func NewContainerWorkflow() *ContainerWorkflow {
	return &ContainerWorkflow{}
}

func (d *ContainerWorkflow) RunContainerDeploymentWorkflow(ctx workflow.Context, deployment *service_model.Deployment) error {
	opts := workflow.ActivityOptions{
		ScheduleToStartTimeout: time.Minute * 10,
		StartToCloseTimeout:    time.Minute * 10,
		TaskQueue:              temporal_constants.NodeTaskQueue,
	}

	ctx = workflow.WithActivityOptions(ctx, opts)

	var containerId string
	err := workflow.ExecuteActivity(ctx, temporal_constants.StartService, deployment).Get(ctx, &containerId)

	if err != nil {
		return err
	}

	log.Info().Str("containerId", containerId).Msg("Service started, waiting for 30 seconds")

	// this is not saved in db yet.
	// we can do that because temporal saves the state of the workflow
	deployment.Execution.ContainerID = containerId
	deployment.Execution.LastLogs = time.Now()

	cleanUpCtx, _ := workflow.NewDisconnectedContext(ctx)

	defer func() {
		err := workflow.ExecuteActivity(cleanUpCtx, temporal_constants.StopService, deployment).Get(cleanUpCtx, nil)

		if err != nil {
			log.Error().Err(err).Str("containerId", containerId).Msg("Error stopping service")
		}

		log.Info().Str("containerId", containerId).Msg("Service stopped")
	}()

	iteration := 0

	for {
		if iteration > 10 {
			break
		}

		workflow.Sleep(ctx, time.Second*5)

		var logs []*brume_logs.Log

		err := workflow.ExecuteActivity(ctx, temporal_constants.GetLogs, deployment).Get(ctx, &logs)

		if len(logs) > 0 {
			log.Info().Str("containerId", containerId).Msgf("Found %d logs", len(logs))

			masterOpts := workflow.ActivityOptions{
				ScheduleToStartTimeout: time.Minute * 10,
				StartToCloseTimeout:    time.Minute * 10,
				TaskQueue:              temporal_constants.MasterTaskQueue,
			}

			masterCtx := workflow.WithActivityOptions(ctx, masterOpts)

			err := workflow.ExecuteActivity(masterCtx, temporal_constants.IngestLogs, logs).Get(masterCtx, nil)

			if err != nil {
				log.Error().Err(err).Str("containerId", containerId).Msg("Error ingesting logs")
			}
		}

		deployment.Execution.LastLogs = time.Now()

		if err != nil {
			log.Error().Err(err).Str("containerId", containerId).Msg("Error getting logs")
		}

		iteration++
	}

	return nil
}
