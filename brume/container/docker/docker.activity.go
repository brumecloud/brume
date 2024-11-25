package docker

import (
	"context"

	service_model "brume.dev/service/model"
	"github.com/rs/zerolog/log"
)

type DockerActivity struct {
	dockerService *DockerService
}

func NewDockerActivity(dockerService *DockerService) *DockerActivity {
	return &DockerActivity{dockerService: dockerService}
}

func (d *DockerActivity) StartService(ctx context.Context, service *service_model.Service) (string, error) {
	log.Info().Str("image", service.LiveBuilder.Data.Image).Str("name", service.Name).Msg("Starting container")

	image, err := d.dockerService.PullImage(service.LiveBuilder.Data.Registry, service.LiveBuilder.Data.Image, service.LiveBuilder.Data.Tag)

	if err != nil {
		return "", err
	}

	containerId, err := d.dockerService.StartContainer(image, service.LiveRunner)

	if err != nil {
		return "", err
	}

	log.Info().Str("containerId", containerId).Msg("Service started")

	return containerId, nil
}

func (d *DockerActivity) StopService(ctx context.Context, service *service_model.Service, containerId string) error {
	log.Info().Str("containerId", containerId).Str("service", service.Name).Msg("Stopping service")

	err := d.dockerService.StopContainer(containerId)

	if err != nil {
		return err
	}

	log.Debug().Str("containerId", containerId).Msg("Removing container")
	err = d.dockerService.RemoveContainer(containerId)

	return err
}
