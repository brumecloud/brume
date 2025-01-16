package job_service

import (
	"context"
	"time"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/db"
	brume_log "brume.dev/internal/log"
	job_model "brume.dev/jobs/model"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
	"go.temporal.io/sdk/client"
)

var logger = brume_log.GetLogger("bid_service")

type BidService struct {
	db             *db.DB
	temporalClient client.Client
}

func NewBidService(db *db.DB, temporalClient client.Client) *BidService {
	return &BidService{db: db, temporalClient: temporalClient}
}

func (s *BidService) CreateBid(deployment *deployment_model.Deployment, serviceID uuid.UUID, workflowID string, runID string) (*job_model.Job, error) {
	logger.Info().Interface("deployment", deployment).Str("service_id", serviceID.String()).Msg("Creating bid")
	bid := &job_model.Job{
		ID:         uuid.New(),
		Deployment: deployment,
		ServiceID:  serviceID,
		Price:      1000,
		WorkflowID: workflowID,
		RunID:      runID,
	}

	logger.Info().Interface("bid", bid).Msg("Creating bid")
	return bid, s.db.Gorm.Create(bid).Error
}

// get all bid
func (s *BidService) GetBidsForProject(projectID string) ([]*job_model.Job, error) {
	var bids []*job_model.Job
	serviceIDs := []string{}

	err := s.db.Gorm.Model(&service_model.Service{}).Where("project_id = ?", projectID).Pluck("id", &serviceIDs).Error
	if err != nil {
		return nil, err
	}

	err = s.db.Gorm.Model(&job_model.Job{}).Where("service_id IN ? AND accepted_at IS NOT NULL", serviceIDs).Find(&bids).Error

	return bids, err
}

func (s *BidService) GetAllCurrentBids() ([]*job_model.Job, error) {
	var bids []*job_model.Job
	err := s.db.Gorm.Model(&job_model.Job{}).Where("accepted_at IS NULL").Find(&bids).Error
	return bids, err
}

// for the moment we accept the first bid
func (s *BidService) AcceptBid(bidID string, machineID uuid.UUID) error {
	logger.Info().Str("bid_id", bidID).Str("machine_id", machineID.String()).Msg("Accepting bid")

	bid := &job_model.Job{}
	err := s.db.Gorm.Model(&job_model.Job{}).Where("id = ?", bidID).First(bid).Error
	if err != nil {
		return err
	}

	now := time.Now()

	bid.AcceptedAt = &now
	bid.MachineID = &machineID

	err = s.db.Gorm.Model(&job_model.Job{}).Where("id = ?", bidID).Updates(bid).Error
	if err != nil {
		return err
	}

	// we need to update the workflow to signal that the machine has been found
	ctxWithTimeout, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	_, err = s.temporalClient.UpdateWorkflow(ctxWithTimeout, client.UpdateWorkflowOptions{
		WorkflowID: bid.WorkflowID,
		RunID:      bid.RunID,
		UpdateName: "machine_found",
	})
	if err != nil {
		logger.Error().Err(err).Str("bid_id", bidID).Str("machine_id", machineID.String()).Msg("Failed to update (machine found) workflow")
		return err
	}

	return nil
}
