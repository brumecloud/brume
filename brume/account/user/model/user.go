package user_model

import (
	brume_utils "brume.dev/internal/utils"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

type User struct {
	gorm.Model
	ID         string `gorm:"type:varchar(255);primaryKey"`
	ProviderID string `gorm:"unique"`
	Name       string
	Password   string
	Avatar     string

	// user only have one organization
	OrganizationID string
}

func (u *User) HashPassword() error {
	hash, err := bcrypt.GenerateFromPassword([]byte(u.Password), bcrypt.DefaultCost)
	if err != nil {
		return err
	}

	u.Password = string(hash)

	return nil
}

func (u *User) CheckPassword(password string) error {
	return bcrypt.CompareHashAndPassword([]byte(u.Password), []byte(password))
}

func (u *User) BeforeCreate(tx *gorm.DB) error {
	u.ID = brume_utils.UserID()
	return u.HashPassword()
}
