package org

import (
	org "brume.dev/account/org/model"
	user "brume.dev/account/user/model"
	"brume.dev/internal/db"
	"github.com/rs/zerolog/log"
)

type OrganizationService struct {
	db *db.DB
}

func NewOrganizationService(db *db.DB) *OrganizationService {
	return &OrganizationService{
		db: db,
	}
}

func (s *OrganizationService) GetUserOrganization(email string) ([]*org.Organization, error) {
	var user *user.User
	err := s.db.Gorm.First(&user, "email = ?", email).Error

	log.Debug().Str("email", user.Email).Msg("get user")

	if err != nil {
		return nil, err
	}

	var orgs []*org.Organization
	err = s.db.Gorm.Find(&orgs, "id = ?", user.OrganizationID).Error

	if err != nil {
		return nil, err
	}

	return orgs, nil
}
