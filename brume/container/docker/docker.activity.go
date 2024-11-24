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

func (d *DockerActivity) StartContainer(ctx context.Context, service *service_model.Service) error {
	log.Info().Str("image", service.Builder.Data.Image).Str("name", service.Name).Msg("Starting container")
	return nil
}
