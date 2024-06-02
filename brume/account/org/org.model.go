package org

import (
	"github.com/brume/brume/account/user"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Organization struct {
	gorm.Model
	ID   uuid.UUID `gorm:"type:uuid;primaryKey"`
	Name string    `gorm:"unique"`

	Users []user.User `gorm:"foreignKey:OrganizationID;references:ID"`
}

func (o *Organization) BeforeCreate(tx *gorm.DB) error {
	o.ID = uuid.New()
	return nil
}
