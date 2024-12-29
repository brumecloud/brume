package bid_workflow

import (
	job_model "brume.dev/internal/jobs/model"
	"github.com/rs/zerolog/log"
	"go.temporal.io/sdk/workflow"
)

type BidWorkflow struct {
}

var logger = log.With().Str("module", "bid_workflow").Logger()

func NewBidWorkflow() *BidWorkflow {
	return &BidWorkflow{}
}

func (b *BidWorkflow) BidWorkflow(ctx workflow.Context, bid *job_model.Job) error {
	logger.Info().Interface("bid", bid).Msg("Starting bid workflow")

	return nil
}
