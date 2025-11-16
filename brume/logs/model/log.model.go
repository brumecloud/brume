package log_model

import (
	"time"
)

// otel log model
type RawLog struct {
	Timestamp     time.Time
	SeverityText  string
	LogAttributes map[string]string
}

type Log struct {
	ContainerID    string    `json:"container_id"`
	Message        string    `json:"body"`
	Level          string    `json:"level"`
	Timestamp      time.Time `json:"timestamp"`
	ServiceID      string    `json:"service_id"`
	DeploymentID   string    `json:"deployment_id"`
	DeploymentName string    `json:"deployment_name"`
}

// this is the format of the logs that the agent sends to the orchestrator
type AgentLogs struct {
	JobID     string `json:"job_id" validate:"required"`
	Message   string `json:"message" validate:"required"`
	Level     string `json:"level" validate:"required"`
	Timestamp string `json:"timestamp" validate:"required"`
}
