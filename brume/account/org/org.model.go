package org

import "gorm.io/gorm"

type Organization struct {
	gorm.Model
	ID   uint   `gorm:"primaryKey"`
	Name string `gorm:"unique"`
}
