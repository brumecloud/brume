package stack_model

import (
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Stack struct {
	gorm.Model
	ID         uuid.UUID `gorm:"type:uuid;primaryKey" json:"id"`
	Name       string    `gorm:"type:varchar(255);not null" json:"name"`
	TemplateID string    `gorm:"type:varchar(255);not null" json:"template_id"`

	CloudAccountID uuid.UUID `gorm:"type:uuid;not null" json:"cloud_account_id"`
}
