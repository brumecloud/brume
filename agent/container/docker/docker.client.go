package docker

import (
	"context"
	"fmt"
	"io"
	"os"
	"strings"
	"time"

	runner_model "brume.dev/runner/model"
	"github.com/brumecloud/agent/internal/log"
	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/filters"
	"github.com/docker/docker/api/types/image"
	"github.com/docker/docker/api/types/strslice"
	"github.com/docker/docker/client"
	"github.com/google/uuid"
	"go.uber.org/fx"
)

// this is the file doing the docker client interaction

var logger = log.GetLogger("docker")

var DockerModule = fx.Module("docker",
	fx.Provide(NewDockerService),
)

type DockerService struct {
	dockerClient *client.Client
}

func NewDockerService() *DockerService {
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		panic(err)
	}

	logger.Info().Msg("Node connected to docker engine")

	return &DockerService{dockerClient: cli}
}

func (d *DockerService) StartContainer(imageId string, jobID uuid.UUID, runner *runner_model.RunnerData) (string, error) {
	logger.Info().Str("imageId", imageId).Str("jobId", jobID.String()).Msg("Starting container")

	ctx := context.Background()
	var command strslice.StrSlice
	if runner.Type != runner_model.RunnerTypeDocker {
		return "", fmt.Errorf("runner is not a docker runner")
	}

	if runner.Docker.Command != "" {
		command = strslice.StrSlice(strings.Split(runner.Docker.Command, " "))
	}

	response, err := d.dockerClient.ContainerCreate(ctx, &container.Config{
		Image: imageId,
		Tty:   false,
		Labels: map[string]string{
			"brume.dev/managed": "true",
			"brume.dev/job-id":  jobID.String(),
		},
		Cmd: command,
		// Healthcheck: &container.HealthConfig{
		// 	Test:     []string{"CMD", "/bin/sh", "curl", runner.Data.HealthCheckURL},
		// 	Interval: 2 * time.Second,
		// 	Timeout:  5 * time.Second,
		// 	Retries:  3,
		// },
	}, nil, nil, nil, "")
	if err != nil {
		return "", err
	}

	logger.Info().Str("containerId", response.ID).Msg("Container started")

	err = d.dockerClient.ContainerStart(ctx, response.ID, container.StartOptions{})
	if err != nil {
		logger.Error().Err(err).Str("containerId", response.ID).Msg("Failed to start container")
		return "", err
	}

	return response.ID, nil
}

func (d *DockerService) StopContainer(containerId string) error {
	logger.Info().Str("containerId", containerId).Msg("Stopping container")

	return d.dockerClient.ContainerStop(context.Background(), containerId, container.StopOptions{})
}

func (d *DockerService) RemoveContainer(containerId string) error {
	return d.dockerClient.ContainerRemove(context.Background(), containerId, container.RemoveOptions{})
}

func (d *DockerService) PullImage(registry string, image_name string, tag string) (string, error) {
	totalImage := fmt.Sprintf("%s/%s:%s", registry, image_name, tag)
	logger.Info().Str("image", totalImage).Msg("Pulling image")
	reader, err := d.dockerClient.ImagePull(context.Background(), totalImage, image.PullOptions{})
	if err != nil {
		return "", err
	}

	defer reader.Close()

	io.Copy(os.Stdout, reader)

	return totalImage, nil
}

func (d *DockerService) GetLogs(containerId string, since time.Time) (io.ReadCloser, error) {
	logger.Info().Str("containerId", containerId).Time("since", since).Msg("Getting logs")

	out, err := d.dockerClient.ContainerLogs(context.Background(), containerId, container.LogsOptions{
		ShowStdout: true, ShowStderr: true, Since: since.Format(time.RFC3339),
	})

	return out, err
}

func (d *DockerService) StatusContainer(containerId string) (*types.ContainerState, error) {
	inspect, err := d.dockerClient.ContainerInspect(context.Background(), containerId)
	if err != nil {
		return nil, err
	}

	return inspect.State, nil
}

func (d *DockerService) GetAllRunningContainers() ([]types.Container, error) {
	list, err := d.dockerClient.ContainerList(context.Background(), container.ListOptions{
		All:     true,
		Filters: filters.NewArgs(filters.Arg("label", "brume.dev/managed"), filters.Arg("status", "running")),
	})
	if err != nil {
		return nil, err
	}

	return list, nil
}
