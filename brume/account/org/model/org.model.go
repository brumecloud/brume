package org_model

import (
	user "brume.dev/account/user/model"
	cloud_account_model "brume.dev/cloud/account/model"
	project "brume.dev/project/model"
	"gorm.io/gorm"
)

type Organization struct {
	gorm.Model
	// the id is the provider id of the organization
	ID string `gorm:"unique;primaryKey;type:varchar(255)"`

	Name string `gorm:"unique"`

	Users         []user.User                        `gorm:"foreignKey:OrganizationID;references:ID"`
	Projects      []*project.Project                 `gorm:"many2many:organization_projects;"`
	CloudAccounts []cloud_account_model.CloudAccount `gorm:"foreignKey:OrganizationID;references:ID"`
}
