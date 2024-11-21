package builder_service

import (
	builder_model "brume.dev/builder/model"
	"brume.dev/internal/db"
	"github.com/google/uuid"
)

type BuilderService struct {
	db *db.DB
}

func NewBuilderService(db *db.DB) *BuilderService {
	return &BuilderService{
		db: db,
	}
}

func (e *BuilderService) GetBuilder(builderId uuid.UUID) (*builder_model.Builder, error) {
	builder := &builder_model.Builder{}
	err := e.db.Gorm.First(builder, builderId).Error
	return builder, err
}

func (e *BuilderService) DuplicateBuilder(builderId uuid.UUID) (*builder_model.Builder, error) {
	builder, err := e.GetBuilder(builderId)

	if err != nil {
		return nil, err
	}

	duplicateBuilder := builder

	id, _ := uuid.NewRandom()
	duplicateBuilder.ID = id

	err = e.db.Gorm.Create(&duplicateBuilder).Error

	return duplicateBuilder, err
}

func (e *BuilderService) CreateDockerBuilder(serviceId uuid.UUID) (*builder_model.Builder, error) {
	id, _ := uuid.NewRandom()

	builder := &builder_model.Builder{
		ID:        id,
		ServiceId: serviceId,
		Type:      "generic-docker",
		Data: builder_model.BuilderData{
			Image:    "nginx",
			Registry: "docker.io",
			Tag:      "latest",
		},
	}

	err := e.db.Gorm.Create(&builder).Error

	return builder, err
}
