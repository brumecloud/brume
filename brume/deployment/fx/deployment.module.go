package fx_deployment

import (
	deployment_workflow "brume.dev/deployment/workflow"
	"go.uber.org/fx"
)

var DeploymentModule = fx.Module("deployment",
	fx.Provide(deployment_workflow.NewDeploymentWorkflow),
	fx.Invoke(func(deploymentWorkflow *deployment_workflow.DeploymentWorkflow) {}),
)
