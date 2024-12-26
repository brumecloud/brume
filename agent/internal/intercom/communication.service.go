// package used to communicate with the orchestrator
package intercom_service

import "github.com/rs/zerolog/log"

type IntercomService struct {
}

func NewIntercomService() *IntercomService {
	return &IntercomService{}
}

func (i *IntercomService) SendGeneralHealth(health bool) {
	log.Info().Bool("health", health).Msg("Sending general health")
}

func (i *IntercomService) SendJobHealth(health map[string]bool) {
	log.Info().Interface("health", health).Msg("Sending job health")
}
