package cloud_account_fx

import (
	cloud_account_service "brume.dev/cloud/account"
	cloud_account_activity "brume.dev/cloud/account/activity"
	cloud_account_repository "brume.dev/cloud/account/repository"
	"go.uber.org/fx"
)

var CloudAccountFxModule = fx.Module("cloud_account_fx",
	fx.Provide(
		cloud_account_service.NewCloudAccountService,
		cloud_account_activity.NewCloudAccountActivity,
		cloud_account_repository.NewCloudAccountRepository,
	),
)
