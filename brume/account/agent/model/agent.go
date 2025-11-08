package agent_model

import (
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type AgentType string

const (
	AgentTypeRunner AgentType = "runner"
	AgentTypeBuilder AgentType = "builder"
)

type Agent struct {
	gorm.Model
	ID uuid.UUID `gorm:"type:uuid;primaryKey"`

	// the API key is used to authenticate the agent with the API
	APIKey string `gorm:"type:varchar(255);not null"`

	// the type of agent
	AgentType AgentType `gorm:"type:varchar(255);not null"`

	// the organization the agent belongs to
	OrganizationID uuid.UUID `gorm:"type:uuid;not null"`
}