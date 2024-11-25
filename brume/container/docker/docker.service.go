package docker

import (
	"context"
	"fmt"
	"io"
	"os"
	"strings"

	runner_model "brume.dev/runner/model"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/image"
	"github.com/docker/docker/api/types/strslice"
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

func (d *DockerService) StartContainer(imageId string, runner *runner_model.Runner) (string, error) {
	log.Info().Str("imageId", imageId).Msg("Starting container")

	ctx := context.Background()
	var command strslice.StrSlice

	if runner.Data.Command != "" {
		command = strslice.StrSlice(strings.Split(runner.Data.Command, " "))
	}

	response, err := d.dockerClient.ContainerCreate(ctx, &container.Config{
		Image: imageId,
		Tty:   false,
		Labels: map[string]string{
			"brume.dev/managed":    "true",
			"brume.dev/service-id": runner.ServiceId.String(),
		},
		Cmd: command,
		// Healthcheck: &container.HealthConfig{
		// 	Test:     []string{"CMD", "/bin/sh", "curl", runner.Data.HealthCheckURL},
		// 	Interval: 2 * time.Second,
		// 	Timeout:  5 * time.Second,
		// 	Retries:  3,
		// },
	}, nil, nil, nil, runner.Name)

	if err != nil {
		return "", err
	}

	log.Info().Str("containerId", response.ID).Msg("Container started")

	err = d.dockerClient.ContainerStart(ctx, response.ID, container.StartOptions{})

	if err != nil {
		log.Error().Err(err).Str("containerId", response.ID).Msg("Failed to start container")
		return "", err
	}

	return response.ID, nil
}

func (d *DockerService) StopContainer(containerId string) error {
	log.Info().Str("containerId", containerId).Msg("Stopping container")

	return d.dockerClient.ContainerStop(context.Background(), containerId, container.StopOptions{})
}

func (d *DockerService) RemoveContainer(containerId string) error {
	return d.dockerClient.ContainerRemove(context.Background(), containerId, container.RemoveOptions{})
}

func (d *DockerService) PullImage(registry string, image_name string, tag string) (string, error) {
	totalImage := fmt.Sprintf("%s/%s:%s", registry, image_name, tag)
	log.Info().Str("image", totalImage).Msg("Pulling image")
	reader, err := d.dockerClient.ImagePull(context.Background(), totalImage, image.PullOptions{})

	if err != nil {
		return "", err
	}

	defer reader.Close()

	io.Copy(os.Stdout, reader)

	return totalImage, nil
}
