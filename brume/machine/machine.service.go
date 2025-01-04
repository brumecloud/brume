package machine

import (
	"context"
	"fmt"
	"time"

	"brume.dev/internal/db"
	machine_model "brume.dev/machine/model"
	"github.com/google/uuid"
	redis "github.com/redis/go-redis/v9"
	"github.com/rs/zerolog/log"
)

var logger = log.With().Str("module", "machine").Logger()

type MachineService struct {
	db    *db.DB
	redis *redis.Client
}

func NewMachineService(db *db.DB, redis *redis.Client) *MachineService {
	return &MachineService{
		db:    db,
		redis: redis,
	}
}

func (e *MachineService) GetMachine(orgId uuid.UUID) ([]*machine_model.Machine, error) {
	machines := []*machine_model.Machine{}
	err := e.db.Gorm.Where("organization_id = ?", orgId).Find(&machines).Error
	return machines, err
}

func (e *MachineService) RecordStatus(machineId uuid.UUID, status string) error {
	logger.Trace().Str("machineId", machineId.String()).Str("status", status).Msg("Recording status")
	err := e.redis.Set(context.Background(), fmt.Sprintf("machine:%s:status", machineId.String()), status, 60*time.Second).Err()
	if err != nil {
		logger.Error().Err(err).Str("machineId", machineId.String()).Str("status", status).Msg("Failed to record status")
		return err
	}
	return nil
}
