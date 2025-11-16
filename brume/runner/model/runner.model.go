package runner_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

	"gorm.io/gorm"
)

type Runner struct {
	ID string `gorm:"type:varchar(255);primaryKey"`

	// reference to the service
	ServiceId string `gorm:"type:varchar(255)"`

	// reference to the builder repository
	// this link must follow the builder convention
	Link string `gorm:"type:text"`
	// builder follow semver
	Version string `gorm:"type:text"`

	Type string

	// we download the json schema from the runner repository
	Schema interface{} `gorm:"type:jsonb"`

	// this data is respecting the schema imposed by the runner
	Data interface{} `gorm:"type:jsonb"`

	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`
}

func (r *Runner) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &r)
}

func (r *Runner) Value() (driver.Value, error) {
	return json.Marshal(r)
}
