package job_model

import (
	"time"

	deployment_model "brume.dev/deployment/model"
	"github.com/google/uuid"
)

type Job struct {
	ID uuid.UUID

	Price int

	CreatedAt  time.Time
	AcceptedAt *time.Time

	WorkflowID string
	RunID      string

	ServiceID  uuid.UUID
	MachineID  *uuid.UUID
	Deployment *deployment_model.Deployment
}
