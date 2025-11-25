package stack_model

import (
	"gorm.io/gorm"
)

type StackStatus string

const (
	StackStatusPending   StackStatus = "Pending"
	StackStatusDeploying StackStatus = "Deploying"
	StackStatusDeployed  StackStatus = "Deployed"
	StackStatusFailed    StackStatus = "Failed"
)

type Stack struct {
	gorm.Model
	ID         string `gorm:"type:varchar(255);primaryKey" json:"id"`
	Name       string `gorm:"type:varchar(255);not null" json:"name"`
	TemplateID string `gorm:"type:varchar(255);not null" json:"template_id"`

	CloudAccountID string      `gorm:"type:varchar(255);not null" json:"cloud_account_id"`
	Status         StackStatus `gorm:"type:varchar(255);not null" json:"status"`
}

type StackTemplate struct {
	gorm.Model
	ID   string `gorm:"type:varchar(255);primaryKey" json:"id"`
	Name string `gorm:"type:varchar(255);not null" json:"name"`
}
