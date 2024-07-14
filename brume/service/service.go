package service

import (
	"brume.dev/internal/db"
	service_model "brume.dev/service/model"
	"github.com/google/uuid"
)

type ServiceService struct {
	db *db.DB
}

func NewServiceService(db *db.DB) *ServiceService {
	return &ServiceService{
		db: db,
	}
}

func (s *ServiceService) CreateService(name string) (*service_model.Service, error) {
	id, _ := uuid.NewRandom()
	service := &service_model.Service{
		Name: name,
		ID:   id,
	}

	err := s.db.Gorm.Create(service).Error

	return service, err
}
