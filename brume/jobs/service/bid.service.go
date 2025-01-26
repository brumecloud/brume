package job_service

import (
	"context"
	"errors"
	"time"

	"brume.dev/internal/db"
	brume_log "brume.dev/internal/log"
	job_model "brume.dev/jobs/model"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
	"go.temporal.io/sdk/client"
)

var bidLogger = brume_log.GetLogger("bid_service")

type BidService struct {
	db             *db.DB
	temporalClient client.Client
}

func NewBidService(db *db.DB, temporalClient client.Client) *BidService {
	return &BidService{db: db, temporalClient: temporalClient}
}

func (s *BidService) UpdateBid(bid *job_model.Job) error {
	return s.db.Gorm.Model(&job_model.Job{}).Where("id = ?", bid.ID).Updates(bid).Error
}

// get all bid
func (s *BidService) GetBidsForProject(projectID string) ([]*job_model.Job, error) {
	var bids []*job_model.Job
	serviceIDs := []string{}

	err := s.db.Gorm.Model(&service_model.Service{}).Where("project_id = ?", projectID).Pluck("id", &serviceIDs).Error
	if err != nil {
		return nil, err
	}

	err = s.db.Gorm.Model(&job_model.Job{}).Where("service_id IN ? AND status = ?", serviceIDs, job_model.JobStatusEnumPending).Find(&bids).Error

	return bids, err
}

func (s *BidService) GetAllCurrentBids() ([]*job_model.Job, error) {
	var bids []*job_model.Job
	err := s.db.Gorm.Model(&job_model.Job{}).Where("accepted_at IS NULL").Find(&bids).Error
	return bids, err
}

// for the moment we accept the first bid
func (s *BidService) AcceptBid(bidID string, machineID uuid.UUID) error {
	bidLogger.Info().Str("bid_id", bidID).Str("machine_id", machineID.String()).Msg("Accepting bid")

	bid := &job_model.Job{}
	err := s.db.Gorm.Model(&job_model.Job{}).Where("id = ?", bidID).First(bid).Error
	if err != nil {
		return err
	}

	now := time.Now()

	bid.AcceptedAt = &now
	bid.MachineID = &machineID
	bid.Status = job_model.JobStatusEnumRunning

	err = s.db.Gorm.Model(&job_model.Job{}).Where("id = ?", bidID).Updates(bid).Error
	if err != nil {
		return err
	}

	// we need to update the workflow to signal that the machine has been found
	ctxWithTimeout, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	bidWorkflowID := bid.BidWorkflowID
	bidRunID := bid.BidRunID
	if bidWorkflowID == nil || bidRunID == nil {
		return errors.New("bid workflow id or run id is nil")
	}

	_, err = s.temporalClient.UpdateWorkflow(ctxWithTimeout, client.UpdateWorkflowOptions{
		WorkflowID:   *bidWorkflowID,
		RunID:        *bidRunID,
		UpdateName:   "machine_found",
		WaitForStage: client.WorkflowUpdateStageAccepted,
		Args:         []interface{}{},
	})
	if err != nil {
		bidLogger.Error().Err(err).Str("bid_id", bidID).Str("machine_id", machineID.String()).Msg("Failed to update (machine found) workflow")
		return err
	}

	return nil
}
