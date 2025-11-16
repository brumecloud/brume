package cloud_account_repository

import (
	"context"
	"errors"

	cloud_account_model "brume.dev/cloud/account/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	brume_utils "brume.dev/internal/utils"
)

var logger = log.GetLogger("cloud.account.repository")

type CloudAccountRepository struct {
	db *db.DB
}

func NewCloudAccountRepository(db *db.DB) *CloudAccountRepository {
	return &CloudAccountRepository{db: db}
}

func (s *CloudAccountRepository) UpdateCloudAccount(ctx context.Context, cloudAccount *cloud_account_model.CloudAccount) (*cloud_account_model.CloudAccount, error) {
	ctx = logger.With().Str("cloud_account_id", cloudAccount.ID).Logger().WithContext(ctx)

	err := s.db.Gorm.Save(cloudAccount).Error
	if err != nil {
		logger.Error().Ctx(ctx).Err(err).Msg("Failed to update cloud account")
		return nil, err
	}

	return cloudAccount, nil
}

func (s *CloudAccountRepository) GetCloudAccountByID(ctx context.Context, id string) (*cloud_account_model.CloudAccount, error) {
	ctx = logger.With().Str("cloud_account_id", id).Logger().WithContext(ctx)

	var cloudAccount cloud_account_model.CloudAccount
	err := s.db.Gorm.First(&cloudAccount, "id = ?", id).Error

	if err != nil {
		logger.Warn().Ctx(ctx).Err(err).Msg("Failed to get cloud account by ID")
		return nil, err
	}

	return &cloudAccount, nil
}

func (s *CloudAccountRepository) CreateCloudAccount(ctx context.Context, cloudAccount *cloud_account_model.CloudAccount) (*cloud_account_model.CloudAccount, error) {
	ctx = logger.With().Str("cloud_account_id", cloudAccount.ID).Logger().WithContext(ctx)
	organizationID := ctx.Value("org_id").(string)

	if cloudAccount.OrganizationID != organizationID {
		logger.Warn().Ctx(ctx).Msg("User not allowed to create cloud account")
		return nil, errors.New("you are not allowed to create this cloud account")
	}

	if cloudAccount.ID == "" {
		cloudAccount.ID = brume_utils.CloudAccountID()
	}

	err := s.db.Gorm.Create(cloudAccount).Error
	if err != nil {
		logger.Error().Ctx(ctx).Err(err).Msg("Failed to create cloud account")
		return nil, err
	}

	return cloudAccount, nil
}

func (s *CloudAccountRepository) PreloadStacks(ctx context.Context, cloudAccount *cloud_account_model.CloudAccount) (*cloud_account_model.CloudAccount, error) {
	ctx = logger.With().Str("cloud_account_id", cloudAccount.ID).Logger().WithContext(ctx)
	err := s.db.Gorm.Preload("Stacks").Find(cloudAccount).Error
	if err != nil {
		logger.Error().Ctx(ctx).Err(err).Msg("Failed to preload stacks")
		return nil, err
	}
	return cloudAccount, nil
}
