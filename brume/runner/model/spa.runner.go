package runner_model

import (
	"database/sql/driver"
	"encoding/json"
)

type SPARunnerData struct {
	IndexFile string `json:"index_file"`
}

func (s *SPARunnerData) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &s)
}

func (s *SPARunnerData) Value() (driver.Value, error) {
	return json.Marshal(s)
}
