package container

import (
	"context"

	"brume.dev/container/runner/docker"
	log_model "brume.dev/logs/model"
	runner_interfaces "brume.dev/runner/interfaces"
	service_model "brume.dev/service/model"
)

// this is the main interface to interact with containers
// all runners with OCI image artifact will use this activity

type ContainerActivity struct {
	containerRunner runner_interfaces.ContainerRunner
}

func NewContainerActivity() *ContainerActivity {
	// TODO based on the node type, factory the right container
	// runner engine. For now, only docker engine is supported
	dockerService := docker.NewDockerService()
	containerRunner := docker.NewDockerEngineRunner(dockerService)

	return &ContainerActivity{
		containerRunner: containerRunner,
	}
}

// start the container from the image
func (c *ContainerActivity) StartService(ctx context.Context, deployment *service_model.Deployment) (string, error) {
	return c.containerRunner.StartService(ctx, deployment)
}

// stop the container
func (c *ContainerActivity) StopService(ctx context.Context, deployment *service_model.Deployment) error {
	return c.containerRunner.StopService(ctx, deployment)
}

// get the logs from the container
func (c *ContainerActivity) GetLogs(ctx context.Context, deployment *service_model.Deployment) ([]*log_model.Log, error) {
	logs, _, err := c.containerRunner.GetLogs(ctx, deployment)

	return logs, err
}
