package org_model

import (
	user "brume.dev/account/user/model"
	cloud_account_model "brume.dev/cloud/account/model"
	project "brume.dev/project/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Organization struct {
	gorm.Model
	ID         uuid.UUID `gorm:"type:uuid;primaryKey"`
	ProviderID string    `gorm:"unique"`
	Name       string    `gorm:"unique"`

	Users         []user.User                        `gorm:"foreignKey:OrganizationID;references:ID"`
	Projects      []*project.Project                 `gorm:"many2many:organization_projects;"`
	CloudAccounts []cloud_account_model.CloudAccount `gorm:"foreignKey:OrganizationID;references:ID"`
}

func (o *Organization) BeforeCreate(tx *gorm.DB) error {
	o.ID = uuid.New()
	return nil
}
