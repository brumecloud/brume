package cloud_aws

import (
	"context"

	brume_config "brume.dev/internal/config"
	"brume.dev/internal/log"
	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/credentials/stscreds"
	"github.com/aws/aws-sdk-go-v2/service/sts"
	"go.uber.org/fx"
)

var logger = log.GetLogger("cloud.aws.service")

var AWSCloudServiceModule = fx.Module("cloud_aws",
	fx.Provide(NewAWSService),
)

type AWSService struct {
	brumeConfig *brume_config.BrumeConfig
}

func NewAWSService(brumeConfig *brume_config.BrumeConfig) *AWSService {
	return &AWSService{
		brumeConfig: brumeConfig,
	}
}

// this will load the default config for **BRUME** aws account and credentials
// this load the creds of the Brume Server.
func (s *AWSService) GetBrumeConfig(ctx context.Context) (*aws.Config, error) {
	cfg, err := config.LoadDefaultConfig(ctx, config.WithRegion("eu-west-3"))

	logger.Debug().Ctx(ctx).Msg("Loading default AWS config")
	if err != nil {
		return nil, err
	}

	return &cfg, nil
}

func (s *AWSService) GetAssumeRoleForBrumeAssume(ctx context.Context) (*aws.Config, error) {
	ctx = logger.With().Str("brume_trust_arn", s.brumeConfig.BrumeGeneralConfig.BrumeTrustArn).Logger().WithContext(ctx)

	cfg, err := s.GetBrumeConfig(ctx)
	if err != nil {
		logger.Error().Ctx(ctx).Err(err).Msg("Failed to load default AWS config")
		return nil, err
	}

	stsClient := sts.NewFromConfig(*cfg)

	creds := stscreds.NewAssumeRoleProvider(stsClient, s.brumeConfig.BrumeGeneralConfig.BrumeTrustArn)

	brumeAssumeConfig := aws.Config{
		Region:      "eu-west-3",
		Credentials: aws.NewCredentialsCache(creds),
	}

	logger.Debug().Ctx(ctx).Interface("brume_assume_config", brumeAssumeConfig.Credentials).Msg("Brume assume config")

	return &brumeAssumeConfig, nil
}

// this will return an aws config from the role arn of the client
func (s *AWSService) GetClientConfigFromAssumeRole(ctx context.Context, roleArn string) (*aws.Config, error) {
	ctx = logger.With().Str("role_arn", roleArn).Logger().WithContext(ctx)

	cfg, err := s.GetAssumeRoleForBrumeAssume(ctx)
	if err != nil {
		logger.Error().Ctx(ctx).Err(err).Msg("Failed to load default AWS config")
		return nil, err
	}

	logger.Debug().Ctx(ctx).Str("role_arn", roleArn).Msg("Getting client config from assume role")

	stsClient := sts.NewFromConfig(*cfg)

	creds := stscreds.NewAssumeRoleProvider(stsClient, roleArn)

	userConfig := aws.Config{
		Region:      "eu-west-3",
		Credentials: aws.NewCredentialsCache(creds),
	}

	return &userConfig, nil
}
