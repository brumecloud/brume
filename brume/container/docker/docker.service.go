package docker

import (
	"github.com/docker/docker/client"
	"github.com/rs/zerolog/log"
)

type DockerService struct {
	dockerClient *client.Client
}

func NewDockerService() *DockerService {
	log.Info().Msg("Connecting to docker client")

	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())

	if err != nil {
		panic(err)
	}

	log.Info().Msg("Connected to docker client")

	return &DockerService{dockerClient: cli}
}

func (d *DockerService) StartContainer(containerName string) error {
	log.Info().Msgf("Starting container %s", containerName)
	return nil
}

func (d *DockerService) StopContainer(containerName string) error {
	log.Info().Msgf("Stopping container %s", containerName)
	return nil
}
