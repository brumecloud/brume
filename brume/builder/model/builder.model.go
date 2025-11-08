package builder_model

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

// builder for a service
type Builder struct {
	ID uuid.UUID `gorm:"type:uuid;primaryKey"`

	// service using this builder
	ServiceId uuid.UUID `gorm:"type:uuid"`

	// reference to the builder repository
	// this link must follow the builder convention
	Link string `gorm:"type:text"`
	// builder follow semver
	Version string `gorm:"type:text"`

	// what kind of artifact the builder produces
	Type      string

	// this data is respecting the schema imposed by the builder
	Data      json.RawMessage `gorm:"type:jsonb"`

	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`
}
