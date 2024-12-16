package deployment_workflow

import (
	deployment_model "brume.dev/deployment/model"

	"go.temporal.io/sdk/workflow"
)

type DeploymentWorkflow struct {
}

func NewDeploymentWorkflow() *DeploymentWorkflow {
	return &DeploymentWorkflow{}
}

func (d *DeploymentWorkflow) RunDeploymentWorkflow(ctx workflow.Context, deployment *deployment_model.Deployment) error {
	return nil
}
