package cloud_account_service

import (
	cloud_account_model "brume.dev/cloud/account/model"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	"go.uber.org/fx"
)

var logger = log.GetLogger("cloud_account_service")

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
