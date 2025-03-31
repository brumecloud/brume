package log_model

import (
	"time"
)

// otel log model
type Log struct {
	Timestamp     time.Time
	SeverityText  string
	LogAttributes map[string]interface{}
}

// this is the format of the logs that the agent sends to the orchestrator
type AgentLogs struct {
	JobID     string `json:"job_id" validate:"required,uuid"`
	Message   string `json:"message" validate:"required"`
	Level     string `json:"level" validate:"required"`
	Timestamp string `json:"timestamp" validate:"required"`
}
