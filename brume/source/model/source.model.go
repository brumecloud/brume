package source_model

import (
	"database/sql/driver"
	"encoding/json"

	"github.com/google/uuid"
)

type SourceType string

const (
	SourceTypeGit SourceType = "git"
)

// this represent what
type Source struct {
	ID uuid.UUID `json:"id"`

	Type SourceType `gorm:"type:text" json:"type"`

	GitData *GitSource `gorm:"type:jsonb" json:"git_data"`

	Link string `gorm:"type:text" json:"link"`
}

func (s *Source) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &s)
}

func (s *Source) Value() (driver.Value, error) {
	return json.Marshal(s)
}

type GitSource struct {
	Provider   string `gorm:"type:text" json:"provider"`
	Repository string `gorm:"type:text" json:"repository"`
	Branch     string `gorm:"type:text" json:"branch"`
}

func (gs *GitSource) Scan(value interface{}) error {
	return json.Unmarshal(value.([]byte), &gs)
}

func (gs *GitSource) Value() (driver.Value, error) {
	return json.Marshal(gs)
}

// Event being created when a source triggers a build (new commit for example)
type SourceEvent struct {
	Source *Source         `json:"source"`
	Git    *GitSourceEvent `json:"git"`
}

type GitSourceEvent struct {
	Commit    string `json:"commit"`
	Message   string `json:"message"`
	Author    string `json:"author"`
	Timestamp string `json:"timestamp"`
}
