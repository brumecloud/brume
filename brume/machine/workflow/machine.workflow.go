package machine_workflow

import (
	machine_model "brume.dev/machine/model"
	"go.temporal.io/sdk/workflow"
)

type MachineWorkflow struct {
}

// for DI
func NewMachineWorkflow() *MachineWorkflow {
	return &MachineWorkflow{}
}

// This workflow is used to run the machine health check.
// This will get all the logs and metrics from the machine and each service running on it.
// This is one of the most important health check of the system
func (m *MachineWorkflow) MachineWorkflow(ctx workflow.Context, machine *machine_model.Machine) error {
	return nil
}
