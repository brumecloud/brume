package project_workflow

import (
	"context"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/log"
)

var logger = log.GetLogger("container_workflow")

type ContainerWorkflow struct{}

func NewContainerWorkflow() *ContainerWorkflow {
	return &ContainerWorkflow{}
}

func (d *ContainerWorkflow) RunContainerDeploymentWorkflow(ctx context.Context, deployment *deployment_model.Deployment) error {

	logger.Info().Str("deployment_id", deployment.ID.String()).Msg("Starting container deployment workflow")

	return nil
}
