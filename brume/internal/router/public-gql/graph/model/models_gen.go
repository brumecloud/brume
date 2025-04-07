// Code generated by github.com/99designs/gqlgen, DO NOT EDIT.

package public_graph_model

type BuilderDataInput struct {
	Image    string `json:"image"`
	Registry string `json:"registry"`
	Tag      string `json:"tag"`
}

type CreateServiceInput struct {
	Name  string `json:"name"`
	Image string `json:"image"`
}

type GetLogsInput struct {
	Since   string   `json:"since"`
	Limit   int      `json:"limit"`
	Filters []string `json:"filters,omitempty"`
}

type Mutation struct {
}

type Query struct {
}

type RessourceConstraintsInput struct {
	Request float64 `json:"request"`
	Limit   float64 `json:"limit"`
}

type RunnerDataInput struct {
	Command        string                     `json:"command"`
	HealthCheckURL string                     `json:"healthCheckURL"`
	Memory         *RessourceConstraintsInput `json:"memory"`
	CPU            *RessourceConstraintsInput `json:"cpu"`
	Port           int                        `json:"port"`
	PublicDomain   string                     `json:"publicDomain"`
	PrivateDomain  string                     `json:"privateDomain"`
}

type ServiceEvent struct {
	ID        string `json:"id"`
	ServiceID string `json:"serviceId"`
	Timestamp string `json:"timestamp"`
	Type      string `json:"type"`
	Data      string `json:"data"`
}

type ServiceSettingsInput struct {
	Name string `json:"name"`
}

type Subscription struct {
}
