package runner_model

import (
	"database/sql/driver"
	"encoding/json"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Runner struct {
	gorm.Model
	ID        uuid.UUID `gorm:"type:uuid;primaryKey"`
	ServiceId uuid.UUID
	Name      string
	Type      string

	Data RunnerData `gorm:"type:jsonb"`
}

type RessourceConstraints struct {
	Request float64
	Limit   float64
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
