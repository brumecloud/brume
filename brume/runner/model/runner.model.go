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

type RunnerType string

const (
	RunnerTypeDocker RunnerType = "docker"
	RunnerTypeStatic RunnerType = "static"
)

// only for docker
type RunnerData struct {
	Type          RunnerType `gorm:"type:text"`
	PublicDomain  string
	PrivateDomain string

	Docker DockerRunnerData `gorm:"type:jsonb"`
}

func (rd *RunnerData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &rd)
}

func (rd *RunnerData) Value() (driver.Value, error) {
	return json.Marshal(rd)
}

type DockerRunnerData struct {
	Command        string
	HealthCheckURL string
	Memory         RessourceConstraints `gorm:"type:jsonb"`
	CPU            RessourceConstraints `gorm:"type:jsonb"`
	Port           int
}

func (rr *DockerRunnerData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &rr)
}

func (rr *DockerRunnerData) Value() (driver.Value, error) {
	return json.Marshal(rr)
}
