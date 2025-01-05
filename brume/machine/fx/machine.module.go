package fx_machine

import (
	"brume.dev/machine"
	machine_workflow "brume.dev/machine/workflow"
	"go.uber.org/fx"
)

var Module = fx.Options(
	fx.Provide(machine.NewMachineService, machine_workflow.NewMachineWorkflow),
	fx.Invoke(func(m *machine.MachineService, w *machine_workflow.MachineWorkflow) {}),
)
