package stack_model

import (
	"gorm.io/gorm"
)

type Stack struct {
	gorm.Model
	ID         string `gorm:"type:varchar(255);primaryKey" json:"id"`
	Name       string `gorm:"type:varchar(255);not null" json:"name"`
	TemplateID string `gorm:"type:varchar(255);not null" json:"template_id"`

	CloudAccountID string `gorm:"type:varchar(255);not null" json:"cloud_account_id"`
}
