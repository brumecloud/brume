package durable

import (
	"context"
	"fmt"
	"log/slog"

	cloud_account_activity "brume.dev/cloud/account/activity"
	aws_cloud_activity "brume.dev/cloud/aws/activity"
	aws_cloud_workflow "brume.dev/cloud/aws/workflow"
	"brume.dev/internal/config"
	"brume.dev/internal/db"
	"brume.dev/internal/log"
	"github.com/rs/zerolog"
	slogzerolog "github.com/samber/slog-zerolog/v2"
	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/worker"
	"go.uber.org/fx"
)

var DurableModule = fx.Module("durable", fx.Provide(NewDurable), fx.Invoke(func(d *Durable) {}))

var logger = log.GetLogger("internal.durable")

type Durable struct {
	config                 *config.BrumeConfig
	awsCloudActivities     *aws_cloud_activity.AWSCloudActivity
	cloudAccountActivities *cloud_account_activity.CloudAccountActivity

	TemporalClient client.Client
}

// we need the db to create the durable database
func NewDurable(lc fx.Lifecycle, config *config.BrumeConfig, db *db.DB, awsCloudActivities *aws_cloud_activity.AWSCloudActivity, cloudAccountActivities *cloud_account_activity.CloudAccountActivity) *Durable {
	logger.Info().Msg("Initializing durable module")
	durable := &Durable{
		config:                 config,
		awsCloudActivities:     awsCloudActivities,
		cloudAccountActivities: cloudAccountActivities,
	}
	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			durable.Start(ctx)
			return nil
		},
	})
	return durable
}

func zerologToSlogLevel(level zerolog.Level) slog.Level {
	switch level {
	case zerolog.DebugLevel:
		return slog.LevelDebug
	case zerolog.InfoLevel:
		return slog.LevelInfo
	case zerolog.WarnLevel:
		return slog.LevelWarn
	case zerolog.ErrorLevel:
		return slog.LevelError
	case zerolog.FatalLevel:
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}

const TemporalTaskQueue = "brume"

func (d *Durable) Start(ctx context.Context) {
	logger.Info().Str("temporal_host", d.config.DurableConfig.TemporalHost).Int("temporal_port", d.config.DurableConfig.TemporalPort).Str("temporal_namespace", d.config.DurableConfig.TemporalNamespace).Msg("Starting durable executor")

	// convert the zerolog logger to a slog logger
	slog := slog.New(slogzerolog.Option{
		Logger: &logger,
		Level:  zerologToSlogLevel(logger.GetLevel()),
	}.NewZerologHandler())

	c, err := client.Dial(client.Options{
		Logger:    slog,
		HostPort:  fmt.Sprintf("%s:%d", d.config.DurableConfig.TemporalHost, d.config.DurableConfig.TemporalPort),
		Namespace: d.config.DurableConfig.TemporalNamespace,
	})

	if err != nil {
		logger.Error().Err(err).Msg("Failed to create temporal client")
		return
	}

	d.TemporalClient = c

	logger.Info().Msg("Temporal client created")

	w := worker.New(c, TemporalTaskQueue, worker.Options{})

	// AWS Cloud Workflow
	w.RegisterWorkflow(aws_cloud_workflow.CreateCloudAccountWorkflow)

	w.RegisterActivity(d.awsCloudActivities.AWS_TestAssumeRole)
	w.RegisterActivity(d.cloudAccountActivities.UpdateCloudAccount)

	go func() {
		err = w.Run(worker.InterruptCh())
		if err != nil {
			logger.Error().Err(err).Msg("Failed to run worker")
			return
		}
	}()

	logger.Info().Msg("Durable executor started")
}
