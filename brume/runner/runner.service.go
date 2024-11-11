package runner

import (
	"brume.dev/internal/db"
	runner "brume.dev/runner/model"
	"github.com/google/uuid"
)

type RunnerService struct {
	db *db.DB
}

func NewRunnerService(db *db.DB) *RunnerService {
	return &RunnerService{
		db: db,
	}
}

func (e *RunnerService) CreateDockerExecutor(image string, serviceId uuid.UUID) (*runner.Runner, error) {
	runner := &runner.Runner{
		Name:      "Docker runner",
		Type:      "docker",
		ServiceId: serviceId,
		Data: runner.RunnerData{
			Command:        "",
			HealthCheckURL: "",
			Memory: runner.RessourceConstraints{
				Request: 100,
				Limit:   100,
			},
			CPU: runner.RessourceConstraints{
				Request: 100,
				Limit:   100,
			},
			Port:          80,
			PublicDomain:  "",
			PrivateDomain: "",
		},
	}

	err := e.db.Gorm.Create(&runner).Error

	return runner, err
}
