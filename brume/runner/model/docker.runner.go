package runner_model

import (
	"database/sql/driver"
	"encoding/json"
)

type DockerRunnerData struct {
	Command        string
	HealthCheckURL string

	Memory RessourceConstraints `gorm:"type:jsonb"`
	CPU    RessourceConstraints `gorm:"type:jsonb"`

	Port int
}

func (rr *DockerRunnerData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &rr)
}

func (rr *DockerRunnerData) Value() (driver.Value, error) {
	return json.Marshal(rr)
}
