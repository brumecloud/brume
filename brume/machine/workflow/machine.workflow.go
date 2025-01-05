package machine_workflow

import (
	"context"
	"fmt"
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
	allMachines, err := m.machineService.GetAllMachines(context.Background())
	if err != nil {
		logger.Error().Err(err).Msg("Failed to get all machines")
		return err
	}

	logger.Info().Int("number_of_machines", len(allMachines)).Msg("Running all of the machine health checks")

	unhealthyMachines := []*machine_model.Machine{}
	healthyMachines := []*machine_model.Machine{}

	// loop over all registered machines and check if they are healthy
	// by querying the redis database
	// if the machine is healthy, the redis key will be set
	// if the machine is unhealthy, the redis key will not be set (TTL will have expired)
	for _, machine := range allMachines {
		_, err := m.redis.Get(context.Background(), fmt.Sprintf("machine:last_alive:%s", machine.ID.String())).Result()
		if err != nil {
			logger.Error().Err(err).Str("machine_id", machine.ID.String()).Msg("Machine not healthy: failed to get last alive")
			unhealthyMachines = append(unhealthyMachines, machine)
		} else {
			healthyMachines = append(healthyMachines, machine)
		}
	}

	logger.Info().Int("number_of_unhealthy_machines", len(unhealthyMachines)).Int("number_of_healthy_machines", len(healthyMachines)).Msg("Unhealthy machines")

	return nil
}

// This workflow is used to run the machine health check.
// This will get all the logs and metrics from the machine and each service running on it.
// This is one of the most important health check of the system
func (m *MachineWorkflow) MachineWorkflow(ctx workflow.Context, machine *machine_model.Machine) error {
	return nil
}
