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

type Workflows struct {
	CreateCloudAccountWorkflowID string `gorm:"type:varchar(255);not null"`
}

type CloudAccount struct {
	gorm.Model

	ID uuid.UUID `gorm:"type:uuid;primaryKey" json:"id"`

	Name        string `gorm:"type:varchar(255);not null" json:"name"`
	Description string `gorm:"type:text;not null" json:"description"`

	CloudProvider CloudProvider `gorm:"type:varchar(255);not null" json:"cloudProvider"`
	Status        CloudStatus   `gorm:"type:varchar(255);not null" json:"status"`

	AWS *AWSCloudAccount `gorm:"jsonb" json:"aws"`

	OrganizationID string `gorm:"type:varchar(255);not null" json:"organizationId"`

	Stacks []*stack_model.Stack `gorm:"foreignKey:CloudAccountID;references:ID" json:"stacks"`

	// workflows are store in a jsonb field for easier querying
	Workflows *Workflows `gorm:"jsonb" json:"workflows"`
}

func (aws *AWSCloudAccount) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), aws)
}

func (aws *AWSCloudAccount) Value() (driver.Value, error) {
	return json.Marshal(aws)
}

func (w *Workflows) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), w)
}

func (w *Workflows) Value() (driver.Value, error) {
	return json.Marshal(w)
}
