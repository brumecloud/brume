package runner

import (
	"brume.dev/internal/db"
	brume_utils "brume.dev/internal/utils"
	runner "brume.dev/runner/model"
)

type RunnerService struct {
	db *db.DB
}

func NewRunnerService(db *db.DB) *RunnerService {
	return &RunnerService{
		db: db,
	}
}

func (e *RunnerService) GetRunner(runnerId string) (*runner.Runner, error) {
	runner := &runner.Runner{}
	err := e.db.Gorm.First(runner, runnerId).Error
	return runner, err
}

func (e *RunnerService) DuplicateRunner(runnerId string) (*runner.Runner, error) {
	runner, err := e.GetRunner(runnerId)
	if err != nil {
		return nil, err
	}

	duplicateRunner := runner

	duplicateRunner.ID = brume_utils.RunnerID()

	err = e.db.Gorm.Create(&duplicateRunner).Error

	return duplicateRunner, err
}

func (e *RunnerService) CreateDockerExecutor(serviceId string) (*runner.Runner, error) {
	runner := &runner.Runner{
		ID:        brume_utils.RunnerID(),
		Type:      "generic-docker",
		ServiceId: serviceId,
		Schema:    nil,
		Data:      nil,
		// Data: runner.RunnerData{
		// 	Type: runner.RunnerTypeDocker,
		// 	Docker: &runner.DockerRunnerData{
		// 		Command:        "",
		// 		HealthCheckURL: "http://localhost:8080/health",
		// 		Memory: runner.RessourceConstraints{
		// 			Request: 100,
		// 			Limit:   100,
		// 		},
		// 		CPU: runner.RessourceConstraints{
		// 			Request: 0.5,
		// 			Limit:   0.5,
		// 		},
		// 		Port: 8080,
		// 	},
		// 	PublicDomain:  "",
		// 	PrivateDomain: nil,
		// },
	}

	err := e.db.Gorm.Create(&runner).Error

	return runner, err
}
