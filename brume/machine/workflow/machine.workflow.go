package machine_workflow

import (
	"context"
	"fmt"

	"brume.dev/internal/log"
	"brume.dev/machine"
	machine_model "brume.dev/machine/model"
	redis "github.com/redis/go-redis/v9"
	"go.uber.org/fx"
)

var logger = log.GetLogger("machine.workflow")

type MachineWorkflow struct {
	machineService *machine.MachineService
	redis          *redis.Client
}

// for DI
func NewMachineWorkflow(lc fx.Lifecycle, machineService *machine.MachineService, redis *redis.Client) *MachineWorkflow {
	m := &MachineWorkflow{
		machineService: machineService,
		redis:          redis,
	}

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			logger.Info().Msg("Starting machine workflow health schedule")
			return nil
		},
		OnStop: func(ctx context.Context) error {
			logger.Info().Msg("Deleting machine health check schedule")
			return nil
		},
	})

	return m
}

func (m *MachineWorkflow) HealthCheck(ctx context.Context) error {
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
		_, err := m.redis.Get(context.Background(), fmt.Sprintf("machine:last_alive:%s", machine.ID)).Result()
		if err != nil {
			logger.Warn().Err(err).Str("machine_id", machine.ID).Msg("Machine not healthy: failed to get last alive")
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
func (m *MachineWorkflow) MachineWorkflow(ctx context.Context, machine *machine_model.Machine) error {
	return nil
}
