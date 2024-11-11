package builder_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Builder struct {
	ServiceId uuid.UUID `gorm:"type:uuid;primaryKey"`
	Type      string
	Data      BuilderData `gorm:"type:jsonb"`

	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`
}

type BuilderData struct {
	Image    string
	Registry string `default:"docker.io"`
	Tag      string `default:"latest"`
}

func (b *BuilderData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &b)
}

func (b *BuilderData) Value() (driver.Value, error) {
	return json.Marshal(b)
}
