package workflow

import (
	"context"

	cloud_account_service "brume.dev/cloud/account"
	cloud_account_model "brume.dev/cloud/account/model"
	aws_cloud_steps "brume.dev/cloud/aws/steps"
	"brume.dev/internal/log"
	"github.com/dbos-inc/dbos-transact-golang/dbos"
	"github.com/google/uuid"
)

var logger = log.GetLogger("cloud.aws.workflow")

type AWSWorkflow struct {
	awsCloudSteps       *aws_cloud_steps.AWSCloudSteps
	cloudAccountService *cloud_account_service.CloudAccountService
}

type CreateCloudAccountWorkflowInput struct {
	OrganizationID uuid.UUID
	AccountID      string
	CloudProvider  cloud_account_model.CloudProvider
	AccountName    string
}

// CreateCloudAccountWorkflow creates a new cloud account for the given organization
// 1. try to access the account in AWS
// 2. try to dry run some ressource creation in AWS
// 3. if successful, create the cloud account in the database
func (w *AWSWorkflow) CreateCloudAccountWorkflow(ctx dbos.DBOSContext, input CreateCloudAccountWorkflowInput) error {
	logger.Info().Msg("Creating cloud account workflow")

	dbos.RunAsStep(ctx, func(ctx context.Context) error {
		return w.awsCloudSteps.TestAWSConnectionStep()
	})

	dbos.RunAsStep(ctx, func(ctx context.Context) error {
		return w.awsCloudSteps.DryRunResourceCreationStep()
	})

	err := dbos.RunAsStep(ctx, func(ctx context.Context) error {
		_, stepErr := w.cloudAccountService.CreateCloudAccount(input)
		return stepErr
	})

	logger.Info().Msg("Cloud account workflow finished")
	return err
}
