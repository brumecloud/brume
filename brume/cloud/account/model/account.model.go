package cloud_account_model

import (
	"database/sql/driver"
	"encoding/json"

	stack_model "brume.dev/stack/model"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type CloudProvider string

const (
	CloudProviderAWS   CloudProvider = "AWS"
	CloudProviderAzure CloudProvider = "Azure"
	CloudProviderGCP   CloudProvider = "GCP"
)

type CloudStatus string

const (
	CloudStatusPending      CloudStatus = "Pending"
	CloudStatusConnected    CloudStatus = "Connected"
	CloudStatusDisconnected CloudStatus = "Disconnected"
	CloudStatusError        CloudStatus = "Error"
)

type AWSCloudAccount struct {
	AccountID string `gorm:"type:text;not null"`
}

type CloudAccount struct {
	gorm.Model

	ID uuid.UUID `gorm:"type:uuid;primaryKey"`

	Name        string `gorm:"type:varchar(255);not null"`
	Description string `gorm:"type:text;not null"`

	CloudProvider CloudProvider `gorm:"type:varchar(255);not null"`
	Status        CloudStatus   `gorm:"type:varchar(255);not null"`

	AWS *AWSCloudAccount `gorm:"jsonb" json:"aws"`

	OrganizationID uuid.UUID `gorm:"type:uuid;not null"`

	Stacks []*stack_model.Stack `gorm:"foreignKey:CloudAccountID;references:ID"`
}

func (aws *AWSCloudAccount) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), aws)
}

func (aws *AWSCloudAccount) Value() (driver.Value, error) {
	return json.Marshal(aws)
}
