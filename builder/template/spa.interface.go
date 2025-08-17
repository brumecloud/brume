package template

type BuilderTemplate interface {
	// check if the template and credentials are valid
	Preview() error

	// deploy the template
	Up() error

	// destroy the template
	Destroy() error
}
