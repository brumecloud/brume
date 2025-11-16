package aws_cloud_activity

import (
	"context"

	cloud_aws "brume.dev/cloud/aws"
	"brume.dev/internal/log"
	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/iam"
	"go.uber.org/fx"
)

var logger = log.GetLogger("cloud.aws.activity")

type AWSCloudActivity struct {
	awsService *cloud_aws.AWSService
}

var AWSCloudActivityModule = fx.Module("aws_cloud_activity",
	fx.Provide(NewAWSCloudActivity),
)

func NewAWSCloudActivity(awsService *cloud_aws.AWSService) *AWSCloudActivity {
	return &AWSCloudActivity{
		awsService: awsService,
	}
}

// the name of the activity to test the AWS connection
// ! must be the same as the function name
const AWS_TestAssumeRoleActivityName = "AWS_TestAssumeRole"

type AWS_TestAssumeRoleInput struct {
	RoleArn string
}

// this will try to assume the role and test listing the role
// in the iam service. need to see the assume role (inception!)
func (s *AWSCloudActivity) AWS_TestAssumeRole(ctx context.Context, input AWS_TestAssumeRoleInput) (bool, error) {
	ctx = logger.With().Str("role_arn", input.RoleArn).Logger().WithContext(ctx)
	logger.Info().Ctx(ctx).Msg("Testing the AWS connection")

	cfg, err := s.awsService.GetClientConfigFromAssumeRole(ctx, input.RoleArn)
	if err != nil {
		logger.Warn().Ctx(ctx).Err(err).Msg("Failed to get client config from assume role")
		return false, err
	}

	iamClient := iam.NewFromConfig(*cfg)

	result, err := iamClient.GetRole(ctx, &iam.GetRoleInput{
		RoleName: aws.String("brume-access"),
	})
	if err != nil {
		logger.Warn().Ctx(ctx).Err(err).Msg("Failed to list roles")
		return false, err
	}

	logger.Debug().Ctx(ctx).Interface("result", result).Msg("Role found")
	return true, nil
}
