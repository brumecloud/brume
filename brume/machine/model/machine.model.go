package machine_model

import (
	org_model "brume.dev/account/org/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Machine struct {
	gorm.Model

	ID   uuid.UUID
	Name string
	IP   string

	// machine are linked to an organization
	OrganizationID uuid.UUID
	Organization   org_model.Organization `gorm:"foreignKey:OrganizationID"`
}
