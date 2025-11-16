package cloud_account_activity

import (
	"context"

	cloud_account_model "brume.dev/cloud/account/model"
	cloud_account_repository "brume.dev/cloud/account/repository"
	"brume.dev/internal/log"
)

var logger = log.GetLogger("cloud.account.activity")

type CloudAccountActivity struct {
	cloudAccountRepository *cloud_account_repository.CloudAccountRepository
}

func NewCloudAccountActivity(cloudAccountRepository *cloud_account_repository.CloudAccountRepository) *CloudAccountActivity {
	return &CloudAccountActivity{cloudAccountRepository: cloudAccountRepository}
}

const UpdateCloudAccountActivityName = "UpdateCloudAccount"

type UpdateCloudAccountInput struct {
	CloudAccount cloud_account_model.CloudAccount `json:"CloudAccount"`
}

func (s *CloudAccountActivity) UpdateCloudAccount(ctx context.Context, input UpdateCloudAccountInput) (*cloud_account_model.CloudAccount, error) {
	return s.cloudAccountRepository.UpdateCloudAccount(ctx, &input.CloudAccount)
}
