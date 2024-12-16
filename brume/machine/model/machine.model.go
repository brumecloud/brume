package machine_model

import (
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Machine struct {
	gorm.Model
	ID uuid.UUID
}
