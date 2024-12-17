package machine

import (
	"brume.dev/internal/db"
	machine_model "brume.dev/machine/model"
	"github.com/google/uuid"
)

type MachineService struct {
	db *db.DB
}

func NewMachineService(db *db.DB) *MachineService {
	return &MachineService{
		db: db,
	}
}

func (e *MachineService) GetMachine(orgId uuid.UUID) ([]*machine_model.Machine, error) {
	machines := []*machine_model.Machine{}
	err := e.db.Gorm.Where("organization_id = ?", orgId).Find(&machines).Error
	return machines, err
}
