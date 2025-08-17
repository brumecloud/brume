package project_workflow

import (
	"context"

	deployment_model "brume.dev/deployment/model"
)

type LogWorkflow struct {
}

func NewLogWorkflow() *LogWorkflow {
	return &LogWorkflow{}
}

func (l *LogWorkflow) RunLogWorkflow(ctx context.Context, deployment *deployment_model.Deployment) error {
	// logs polling will be done by brume cloud
	logger.Info().Str("deployment_id", deployment.ID.String()).Msg("Starting log workflow")

	return nil
}
