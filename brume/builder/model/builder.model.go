package builder_model

import (
	"time"

	source_model "brume.dev/source/model"
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
	Type string

	// we download the json schema from the builder repository
	Schema interface{} `gorm:"type:jsonb"`

	// this data is respecting the schema imposed by the builder
	Data interface{} `gorm:"type:jsonb"`

	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`
}

// Model used when a build request is need.
// This is the model returned to the builder agent
type BuildJobRequest struct {
	Builder     *Builder                  `json:"builder"`
	SourceEvent *source_model.SourceEvent `json:"source_event"`
}
