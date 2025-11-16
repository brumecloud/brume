package aws_cloud_workflow

import (
	"time"

	cloud_account_activity "brume.dev/cloud/account/activity"
	cloud_account_model "brume.dev/cloud/account/model"
	aws_cloud_activity "brume.dev/cloud/aws/activity"
	"brume.dev/internal/log"
	"go.temporal.io/sdk/temporal"
	"go.temporal.io/sdk/workflow"
)

var logger = log.GetLogger("cloud.aws.workflow")

type CreateCloudAccountWorkflowInput struct {
	CloudAccount *cloud_account_model.CloudAccount
}

// CreateCloudAccountWorkflow creates a new cloud account for the given organization
// 1. try to access the account in AWS
// 2. try to dry run some ressource creation in AWS
// 3. if successful, create the cloud account in the database
func CreateCloudAccountWorkflow(ctx workflow.Context, input CreateCloudAccountWorkflowInput) (*cloud_account_model.CloudAccount, error) {
	logger.Info().Msg("Creating cloud account workflow")
	cloudAccount := input.CloudAccount

	ao := workflow.ActivityOptions{
		StartToCloseTimeout: time.Second * 10,
		RetryPolicy: &temporal.RetryPolicy{
			MaximumAttempts:    3,
			MaximumInterval:    time.Second * 10,
			BackoffCoefficient: 2,
		},
	}
	ctx = workflow.WithActivityOptions(ctx, ao)

	// brume-access is the name of the role defined in the cloudformation template
	// this JSON is stored in the cloud/account/utils/cloudformation-access-policy.json file
	roleArn := "arn:aws:iam::" + cloudAccount.AWS.AccountID + ":role/brume-access"

	err := workflow.ExecuteActivity(ctx, aws_cloud_activity.AWS_TestAssumeRoleActivityName, aws_cloud_activity.AWS_TestAssumeRoleInput{
		RoleArn: roleArn,
	}).Get(ctx, nil)

	if err != nil {
		logger.Error().Str("account_id", cloudAccount.AWS.AccountID).Err(err).Msg("Failed to test AWS connection")
		return nil, err
	}

	cloudAccount.Status = cloud_account_model.CloudStatusConnected

	err = workflow.ExecuteActivity(ctx, cloud_account_activity.UpdateCloudAccountActivityName, cloud_account_activity.UpdateCloudAccountInput{
		CloudAccount: *cloudAccount,
	}).Get(ctx, nil)

	if err != nil {
		logger.Error().Err(err).Msg("Failed to update cloud account")
		return nil, err
	}

	logger.Info().Str("cloud_account_id", cloudAccount.ID).Msg("Inital cloud sync finished. Account connected")

	return nil, nil
}
