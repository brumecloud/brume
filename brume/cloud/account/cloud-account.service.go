package cloud_account_service

import (
	"errors"

	cloud_account_model "brume.dev/cloud/account/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	"github.com/google/uuid"
	"go.uber.org/fx"
)

var logger = log.GetLogger("cloud.account")

var CloudAccountServiceModule = fx.Module("cloud_account_service",
	fx.Provide(NewCloudAccountService),
)

type CloudAccountService struct {
	db *db.DB
}

func NewCloudAccountService(db *db.DB) *CloudAccountService {
	return &CloudAccountService{
		db: db,
	}
}

func (s *CloudAccountService) WithStacks(cloudAccount *cloud_account_model.CloudAccount) (*cloud_account_model.CloudAccount, error) {
	err := s.db.Gorm.Preload("Stacks").Find(&cloudAccount).Error
	logger.Info().Str("cloud_account_id", cloudAccount.ID.String()).Msg("Getting stacks for cloud account")
	if err != nil {
		logger.Error().Str("cloud_account_id", cloudAccount.ID.String()).Err(err).Msg("Error getting stacks for cloud account")
		return nil, err
	}
	return cloudAccount, nil
}

type CreateCloudAccountInput struct {
	Name           string
	AccountID      string
	CloudProvider  cloud_account_model.CloudProvider
	OrganizationID uuid.UUID
}

func (s *CloudAccountService) CreateCloudAccount(input CreateCloudAccountInput) (*cloud_account_model.CloudAccount, error) {
	if input.CloudProvider != cloud_account_model.CloudProviderAWS {
		logger.Warn().Str("cloud_provider", string(input.CloudProvider)).Msg("Trying to create cloud account with unsupported cloud provider")
		return nil, errors.New("cloud provider not supported")
	}

	awsCloudAccount := &cloud_account_model.AWSCloudAccount{
		AccountID: input.AccountID,
	}

	cloudAccount := &cloud_account_model.CloudAccount{
		Name:           input.Name,
		CloudProvider:  input.CloudProvider,
		AWS:            awsCloudAccount,
		Status:         cloud_account_model.CloudStatusPending,
		OrganizationID: input.OrganizationID,
	}

	logger.Debug().Interface("cloud_account", cloudAccount).Msg("Creating cloud account")

	err := s.db.Gorm.Create(cloudAccount).Error
	return cloudAccount, err
}
