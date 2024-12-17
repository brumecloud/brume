package fx_machine

import (
	"brume.dev/machine"
	"go.uber.org/fx"
)

var Module = fx.Options(
	fx.Provide(machine.NewMachineService),
	fx.Invoke(func(e *machine.MachineService) {}),
)
