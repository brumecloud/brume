package project

import "github.com/google/uuid"

type ProjectActivity struct {
	projectService *ProjectService
}

func NewProjectActivity(projectService *ProjectService) *ProjectActivity {
	return &ProjectActivity{projectService: projectService}
}

func (p *ProjectActivity) PushEvent(projectId uuid.UUID, eventType string, eventData interface{}) error {
	return p.projectService.PushEvent(projectId, eventType, eventData)
}
