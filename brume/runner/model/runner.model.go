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

func (r *Runner) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &r)
}

func (r *Runner) Value() (driver.Value, error) {
	return json.Marshal(r)
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
	Type          RunnerType
	PublicDomain  string
	PrivateDomain string

	Docker *DockerRunnerData
	Static *StaticRunnerData
}

func (rd *RunnerData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &rd)
}

func (rd *RunnerData) Value() (driver.Value, error) {
	return json.Marshal(rd)
}

type StaticRunnerData struct {
	Path string
}

func (sr *StaticRunnerData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &sr)
}

func (sr *StaticRunnerData) Value() (driver.Value, error) {
	return json.Marshal(sr)
}

type DockerRunnerData struct {
	Command        string
	HealthCheckURL string
	Memory         RessourceConstraints
	CPU            RessourceConstraints
	Port           int
}

func (rr *DockerRunnerData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &rr)
}

func (rr *DockerRunnerData) Value() (driver.Value, error) {
	return json.Marshal(rr)
}
