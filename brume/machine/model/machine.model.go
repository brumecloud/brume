package machine_model

import (
	org_model "brume.dev/account/org/model"
	"gorm.io/gorm"
)

type Machine struct {
	gorm.Model

	ID   string
	Name string
	IP   string

	// machine are linked to an organization
	// the brume machine will not tho, this might be an edge case ?
	OrganizationID string
	Organization   org_model.Organization `gorm:"foreignKey:OrganizationID"`
}
