package job_service

import (
	"time"

	deployment_model "brume.dev/deployment/model"
	"brume.dev/internal/db"
	job_model "brume.dev/internal/jobs/model"
	service_model "brume.dev/service/model"
	"github.com/rs/zerolog/log"
)

var logger = log.With().Str("module", "bid_service").Logger()

type BidService struct {
	db *db.DB
}

func NewBidService(db *db.DB) *BidService {
	return &BidService{db: db}
}

func (s *BidService) CreateBid(deployment *deployment_model.Deployment, service *service_model.Service) error {
	logger.Info().Interface("deployment", deployment).Interface("service", service).Msg("Creating bid")
	bid := &job_model.Job{
		Deployment: deployment,
		ServiceID:  service.ID.String(),
		Price:      1000,
	}

	logger.Info().Interface("bid", bid).Msg("Creating bid")
	return s.db.Gorm.Create(bid).Error
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

	return bids, nil
}

func (s *BidService) AcceptBid(bidID string, machineID string) error {
	logger.Info().Str("bid_id", bidID).Str("machine_id", machineID).Msg("Accepting bid")
	now := time.Now()
	return s.db.Gorm.Model(&job_model.Job{}).Where("id = ?", bidID).Updates(job_model.Job{
		AcceptedAt: &now,
		MachineID:  &machineID,
	}).Error
}
