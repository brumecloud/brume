package executor

import (
	executor "brume.dev/executor/model"
	"brume.dev/internal/db"
	"github.com/google/uuid"
)

type Executor interface {
	Init()
	Run()
	Kill()
	Logs()
	Metrics()
	Check() (bool, error)
	CheckExecutor() (bool, error)
}

type ExecutorService struct {
	db *db.DB
}

func NewExecutorService(db *db.DB) *ExecutorService {
	return &ExecutorService{
		db: db,
	}
}

func (e *ExecutorService) CreateDockerExecutor(image string) (*executor.Executor, error) {
	id, randomErr := uuid.NewRandom()

	if randomErr != nil {
		return nil, randomErr
	}

	executor := &executor.Executor{
		ID:    id,
		Name:  "Docker executor",
		Type:  "docker",
		Image: image,
	}

	err := e.db.Gorm.Create(&executor).Error

	return executor, err
}
