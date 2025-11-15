package aws_cloud_steps

import (
	"brume.dev/internal/log"
	"go.uber.org/fx"
)

var logger = log.GetLogger("cloud.aws.steps")

type AWSCloudSteps struct{}

var AWSCloudStepsModule = fx.Module("aws_cloud_steps",
	fx.Provide(NewAWSCloudSteps),
)

func NewAWSCloudSteps() *AWSCloudSteps {
	return &AWSCloudSteps{}
}

func TestAWSConnectionStep() error {
	logger.Info().Msg("Testing the AWS connection")

	return nil
}

func DryRunResourceCreationStep() error {
	logger.Info().Msg("Dry running the resource creation")

	return nil
}
