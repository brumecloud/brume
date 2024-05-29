package user

import "gorm.io/gorm"

type User struct {
	gorm.Model
	ID       uint   `gorm:"primaryKey"`
	Email    string `gorm:"unique"`
	Name     string
	Password string
}
