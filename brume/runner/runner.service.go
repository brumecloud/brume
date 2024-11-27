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

func (e *RunnerService) GetRunner(runnerId uuid.UUID) (*runner.Runner, error) {
	runner := &runner.Runner{}
	err := e.db.Gorm.First(runner, runnerId).Error
	return runner, err
}

func (e *RunnerService) DuplicateRunner(runnerId uuid.UUID) (*runner.Runner, error) {
	runner, err := e.GetRunner(runnerId)

	if err != nil {
		return nil, err
	}

	duplicateRunner := runner

	id, _ := uuid.NewRandom()
	duplicateRunner.ID = id

	err = e.db.Gorm.Create(&duplicateRunner).Error

	return duplicateRunner, err
}

func (e *RunnerService) CreateDockerExecutor(serviceId uuid.UUID) (*runner.Runner, error) {
	id, _ := uuid.NewRandom()

	runner := &runner.Runner{
		ID:        id,
		Name:      "Docker runner",
		Type:      "generic-docker",
		ServiceId: serviceId,
		Data: runner.RunnerData{
			Command:        "",
			HealthCheckURL: "http://localhost:8080/health",
			Memory: runner.RessourceConstraints{
				Request: 100,
				Limit:   100,
			},
			CPU: runner.RessourceConstraints{
				Request: 0.5,
				Limit:   0.5,
			},
			Port:          8080,
			PublicDomain:  "",
			PrivateDomain: "",
		},
	}

	err := e.db.Gorm.Create(&runner).Error

	return runner, err
}
