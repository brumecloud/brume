package builder_service

import (
	builder_model "brume.dev/builder/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	"github.com/google/uuid"
)

var logger = log.GetLogger("builder")

type BuilderService struct {
	db *db.DB
}

func NewBuilderService(db *db.DB) *BuilderService {
	return &BuilderService{
		db: db,
	}
}

func (e *BuilderService) GetBuilder(builderId uuid.UUID) (*builder_model.Builder, error) {
	logger.Trace().Str("builder_id", builderId.String()).Msg("Getting builder")

	builder := &builder_model.Builder{}
	err := e.db.Gorm.First(builder, builderId).Error

	if err != nil {
		logger.Error().Err(err).Str("builder_id", builderId.String()).Msg("Error getting builder")
		return nil, err
	}

	return builder, nil
}

func (e *BuilderService) DuplicateBuilder(builderId uuid.UUID) (*builder_model.Builder, error) {
	logger.Trace().Str("builder_id", builderId.String()).Msg("Duplicating builder")

	builder, err := e.GetBuilder(builderId)

	if err != nil {
		logger.Error().Err(err).Str("builder_id", builderId.String()).Msg("Error getting builder")
		return nil, err
	}

	duplicateBuilder := builder

	id, _ := uuid.NewRandom()
	duplicateBuilder.ID = id

	err = e.db.Gorm.Create(&duplicateBuilder).Error

	if err != nil {
		logger.Error().Err(err).Str("builder_id", builderId.String()).Msg("Error duplicating builder")
		return nil, err
	}

	return duplicateBuilder, nil
}

func (e *BuilderService) CreateDockerBuilder(serviceId uuid.UUID) (*builder_model.Builder, error) {
	logger.Trace().Str("service_id", serviceId.String()).Msg("Creating docker builder")

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

	if err != nil {
		logger.Error().Err(err).Str("service_id", serviceId.String()).Msg("Error creating docker builder")
		return nil, err
	}

	return builder, nil
}
