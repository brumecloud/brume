package builder_model

import (
	"database/sql/driver"
	"encoding/json"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Builder struct {
	gorm.Model

	ServiceId uuid.UUID
	Type      string
	Data      BuilderData `gorm:"type:jsonb"`
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
