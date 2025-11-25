package durable

//!! DO NEVER IMPORT ME ANYWHERE
//!! THIS CODE WILL CAUSE A CIRCULAR IMPORT

import (
	"context"

	cloud_account_activity "brume.dev/cloud/account/activity"
	aws_cloud_activity "brume.dev/cloud/aws/activity"
	aws_cloud_workflow "brume.dev/cloud/aws/workflow"
	"brume.dev/internal/config"
	"brume.dev/internal/db"
	temporal_client "brume.dev/internal/durable/client"
	"brume.dev/internal/log"
	stack_activities "brume.dev/stack/durable/activities"
	"go.temporal.io/sdk/worker"
	"go.uber.org/fx"
)

var DurableModule = fx.Module("durable", fx.Provide(NewDurable), fx.Invoke(func(d *Durable) {}))

var logger = log.GetLogger("internal.durable")

type Durable struct {
	config                 *config.BrumeConfig
	awsCloudActivities     *aws_cloud_activity.AWSCloudActivity
	cloudAccountActivities *cloud_account_activity.CloudAccountActivity
	stackActivities        *stack_activities.StackActivities
	temporalClient         *temporal_client.TemporalClient
}

// we need the db to create the durable database
func NewDurable(lc fx.Lifecycle, temporalClient *temporal_client.TemporalClient, config *config.BrumeConfig, db *db.DB, awsCloudActivities *aws_cloud_activity.AWSCloudActivity, cloudAccountActivities *cloud_account_activity.CloudAccountActivity, stackActivities *stack_activities.StackActivities) *Durable {
	logger.Info().Msg("Initializing durable module")
	durable := &Durable{
		config:                 config,
		awsCloudActivities:     awsCloudActivities,
		cloudAccountActivities: cloudAccountActivities,
		stackActivities:        stackActivities,
		temporalClient:         temporalClient,
	}
	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			durable.Start(ctx)
			return nil
		},
	})
	return durable
}

const TemporalTaskQueue = "brume"

func (d *Durable) Start(ctx context.Context) {
	logger.Info().Str("temporal_host", d.config.DurableConfig.TemporalHost).Int("temporal_port", d.config.DurableConfig.TemporalPort).Str("temporal_namespace", d.config.DurableConfig.TemporalNamespace).Msg("Starting durable executor")

	w := worker.New(d.temporalClient.Client, TemporalTaskQueue, worker.Options{})

	// AWS Cloud Workflow
	w.RegisterWorkflow(aws_cloud_workflow.CreateCloudAccountWorkflow)

	w.RegisterActivity(d.awsCloudActivities.AWS_TestAssumeRole)
	w.RegisterActivity(d.cloudAccountActivities.UpdateCloudAccount)
	w.RegisterActivity(d.stackActivities.UpdateStackStatus)

	go func() {
		err := w.Run(worker.InterruptCh())
		if err != nil {
			logger.Error().Err(err).Msg("Failed to run worker")
			return
		}
	}()

	logger.Info().Msg("Durable executor started")
}
