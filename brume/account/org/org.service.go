package org

import "brume.dev/internal/db"

type OrganizationService struct {
	db *db.DB
}

func NewOrganizationService(db *db.DB) *OrganizationService {
	return &OrganizationService{
		db: db,
	}
}
