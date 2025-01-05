package machine_workflow

import (
	"context"
	"time"

	"brume.dev/internal/log"
	temporal_constants "brume.dev/internal/temporal/constants"
	"brume.dev/machine"
	machine_model "brume.dev/machine/model"
	redis "github.com/redis/go-redis/v9"
	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/workflow"
	"go.uber.org/fx"
)

var logger = log.GetLogger("machine_workflow")

type MachineWorkflow struct {
	machineService *machine.MachineService
	temporalClient client.Client
	redis          *redis.Client
}

// for DI
func NewMachineWorkflow(lc fx.Lifecycle, machineService *machine.MachineService, temporalClient client.Client, redis *redis.Client) *MachineWorkflow {
	m := &MachineWorkflow{
		machineService: machineService,
		temporalClient: temporalClient,
		redis:          redis,
	}

	var workflowHandle client.ScheduleHandle
	var err error

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			logger.Info().Msg("Starting machine workflow health schedule")

			scheduleId := "machine-health-check-schedule"

			scheduleOptions := client.ScheduleOptions{
				ID: scheduleId,
				Spec: client.ScheduleSpec{
					Intervals: []client.ScheduleIntervalSpec{
						{
							Every: time.Second * 10,
						},
					},
				},
				Action: &client.ScheduleWorkflowAction{
					Workflow:  temporal_constants.MachineHealthCheck,
					TaskQueue: temporal_constants.MasterTaskQueue,
				},
			}
			workflowHandle, err = temporalClient.ScheduleClient().Create(ctx, scheduleOptions)

			if err != nil || workflowHandle == nil {
				// if the schedule already exists, we don't need to create it again
				if err.Error() == "schedule with this ID is already registered" {
					logger.Warn().Msg("Machine health check schedule already exists")
					return nil
				}

				logger.Error().Err(err).Msg("Failed to create machine health check schedule")
				return err
			}

			return nil
		},
		OnStop: func(ctx context.Context) error {
			logger.Info().Msg("Deleting machine health check schedule")

			if workflowHandle == nil {
				logger.Error().Msg("Workflow handle is nil")
				return nil
			}

			err := workflowHandle.Delete(ctx)
			if err != nil {
				logger.Error().Err(err).Msg("Failed to delete machine health check schedule")
				return err
			}

			return nil
		},
	})

	return m
}

func (m *MachineWorkflow) HealthCheck(ctx workflow.Context) error {
	logger.Info().Msg("Running all of the machine health checks")
	return nil
}

// This workflow is used to run the machine health check.
// This will get all the logs and metrics from the machine and each service running on it.
// This is one of the most important health check of the system
func (m *MachineWorkflow) MachineWorkflow(ctx workflow.Context, machine *machine_model.Machine) error {
	return nil
}
