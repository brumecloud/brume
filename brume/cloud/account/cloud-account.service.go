package cloud_account_service

import (
	"context"
	"errors"

	cloud_account_model "brume.dev/cloud/account/model"
	cloud_account_repository "brume.dev/cloud/account/repository"
	aws_cloud_workflow "brume.dev/cloud/aws/workflow"
	"brume.dev/internal/db"
	"brume.dev/internal/durable"
	"brume.dev/internal/log"
	"go.temporal.io/sdk/client"
)

var logger = log.GetLogger("cloud.account")

type CloudAccountService struct {
	cloudAccountRepository *cloud_account_repository.CloudAccountRepository
	durable                *durable.Durable
}

func NewCloudAccountService(db *db.DB, durable *durable.Durable, cloudAccountRepository *cloud_account_repository.CloudAccountRepository) *CloudAccountService {
	return &CloudAccountService{
		cloudAccountRepository: cloudAccountRepository,
		durable:                durable,
	}
}

// this will preload the stacks for the cloud account
func (s *CloudAccountService) WithStacks(ctx context.Context, cloudAccount *cloud_account_model.CloudAccount) (*cloud_account_model.CloudAccount, error) {
	return s.cloudAccountRepository.PreloadStacks(ctx, cloudAccount)
}

type CreateCloudAccountInput struct {
	Name          string
	AccountID     string
	CloudProvider cloud_account_model.CloudProvider
}

// this will begin the creation of a new cloud account
// this launchs the workflow to create the cloud account
func (s *CloudAccountService) BeginCreateCloudAccount(ctx context.Context, input CreateCloudAccountInput) (*cloud_account_model.CloudAccount, error) {
	organizationID := ctx.Value("org_id").(string)

	ctx = logger.With().Str("account_id", input.AccountID).Str("cloud_provider", string(input.CloudProvider)).Logger().WithContext(ctx)

	if input.CloudProvider != cloud_account_model.CloudProviderAWS {
		logger.Error().Ctx(ctx).Msg("Cloud provider not supported")
		return nil, errors.New("cloud provider not supported")
	}

	awsCloudAccount := &cloud_account_model.AWSCloudAccount{
		AccountID: input.AccountID,
	}

	cloudAccountInput := &cloud_account_model.CloudAccount{
		Name:           input.Name,
		CloudProvider:  input.CloudProvider,
		AWS:            awsCloudAccount,
		Status:         cloud_account_model.CloudStatusPending,
		OrganizationID: organizationID,
	}

	cloudAccount, err := s.cloudAccountRepository.CreateCloudAccount(ctx, cloudAccountInput)
	if err != nil {
		logger.Error().Ctx(ctx).Err(err).Msg("Failed to create cloud account")
		return nil, err
	}

	// start the enrollment workflow
	opts := client.StartWorkflowOptions{
		ID:        "create-cloud-account-workflow-" + cloudAccount.ID.String(),
		TaskQueue: durable.TemporalTaskQueue,
	}

	_, err = s.durable.TemporalClient.ExecuteWorkflow(context.Background(), opts, aws_cloud_workflow.CreateCloudAccountWorkflow, aws_cloud_workflow.CreateCloudAccountWorkflowInput{
		CloudAccount: cloudAccount,
	})

	// but return the id instantly
	return cloudAccount, err
}
