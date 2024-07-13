package fx_project

import (
	"brume.dev/project"
	"go.uber.org/fx"
)

var ProjectModule = fx.Options(
	fx.Provide(project.NewProjectService),
	fx.Invoke(func(s *project.ProjectService) {}),
)
