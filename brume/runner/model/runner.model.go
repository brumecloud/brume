package runner_model

import (
	"database/sql/driver"
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Runner struct {
	ID uuid.UUID `gorm:"type:uuid;primaryKey"`

	// reference to the service
	ServiceId uuid.UUID `gorm:"type:uuid"`

	Name string
	Type string

	Data RunnerData `gorm:"type:jsonb"`

	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`
}

type RessourceConstraints struct {
	Request float64 `json:"request"`
	Limit   float64 `json:"limit"`
}

type RunnerData struct {
	Command        string
	HealthCheckURL string
	Memory         RessourceConstraints
	CPU            RessourceConstraints
	Port           int
	PublicDomain   string
	PrivateDomain  string
}

func (r *RunnerData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &r)
}

func (r *RunnerData) Value() (driver.Value, error) {
	return json.Marshal(r)
}
