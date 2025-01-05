package machine

import (
	"context"
	"fmt"
	"time"

	"brume.dev/internal/db"
	"brume.dev/internal/log"
	machine_model "brume.dev/machine/model"
	"github.com/google/uuid"
	redis "github.com/redis/go-redis/v9"
)

var logger = log.GetLogger("machine")

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

func (e *MachineService) GetAllMachines(ctx context.Context) ([]*machine_model.Machine, error) {
	machines := []*machine_model.Machine{}
	err := e.db.Gorm.Find(&machines).Error
	return machines, err
}

// the machine will also record the status of the job inside
// but not now
func (e *MachineService) RecordStatus(machineId uuid.UUID, status string) error {
	logger.Trace().Str("machineId", machineId.String()).Str("status", status).Msg("Recording status")

	// the last alive is 10s TTL, that way if we find nothing we can put the machine on the unhealth checklist
	err := e.redis.Set(context.Background(), fmt.Sprintf("machine:last_alive:%s", machineId.String()), time.Now().Unix(), 10*time.Second).Err()
	if err != nil {
		logger.Error().Err(err).Str("machineId", machineId.String()).Str("status", status).Msg("Failed to record status")
		return err
	}
	return nil
}
