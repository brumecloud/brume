package fx_deployment

import (
	deployment_service "brume.dev/deployment"
	deployment_workflow "brume.dev/deployment/workflow"
	"go.uber.org/fx"
)

var DeploymentModule = fx.Module("deployment",
	fx.Provide(deployment_workflow.NewDeploymentWorkflow),
	fx.Provide(deployment_service.NewDeploymentService),
	fx.Invoke(func(deploymentWorkflow *deployment_workflow.DeploymentWorkflow, deploymentService *deployment_service.DeploymentService) {
	}),
)
