package job_model

import (
	"time"

	deployment_model "brume.dev/deployment/model"
)

type Job struct {
	ID string

	Price      int
	CreatedAt  time.Time
	AcceptedAt *time.Time

	ServiceID  string
	MachineID  *string
	Deployment *deployment_model.Deployment
}
