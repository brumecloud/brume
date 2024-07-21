package runner

import (
	"brume.dev/internal/db"
	runner "brume.dev/runner/model"
	"github.com/google/uuid"
)

type Runner interface {
	Init()
	Run()
	Kill()
	Logs()
	Metrics()
	Check() (bool, error)
	CheckRunner() (bool, error)
}

type RunnerService struct {
	db *db.DB
}

func NewRunnerService(db *db.DB) *RunnerService {
	return &RunnerService{
		db: db,
	}
}

func (e *RunnerService) CreateDockerExecutor(image string, serviceId uuid.UUID) (*runner.Runner, error) {
	id, randomErr := uuid.NewRandom()

	if randomErr != nil {
		return nil, randomErr
	}

	runner := &runner.Runner{
		ID:        id,
		Name:      "Docker runner",
		Type:      "docker",
		ServiceId: serviceId,
		Image:     image,
	}

	err := e.db.Gorm.Create(&runner).Error

	return runner, err
}
