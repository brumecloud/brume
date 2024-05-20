package org

import "gorm.io/gorm"

type Organization struct {
	gorm.Model
	ID   uint   `gorm:"primaryKey"`
	name string `gorm:"unique"`
}
