// package used to communicate with the orchestrator
package intercom_service

import (
	"bytes"
	"encoding/json"
	"net/http"

	"github.com/brumecloud/agent/internal/config"

	"github.com/rs/zerolog/log"
)

var logger = log.With().Str("module", "intercom").Logger()

type IntercomService struct {
	cfg *config.AgentConfig
}

func NewIntercomService(cfg *config.AgentConfig) *IntercomService {
	return &IntercomService{
		cfg: cfg,
	}
}

func (i *IntercomService) SendGeneralHealth(health string) error {
	logger.Trace().Str("health", health).Msg("Sending general health")

	// do HTTP call to the orchestrator
	jsonData, err := json.Marshal(map[string]interface{}{
		"machine_id": i.cfg.AgentID,
		"status":     health,
	})
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal health data")
		return err
	}

	body := bytes.NewBuffer(jsonData)

	req, err := http.NewRequest(
		"POST",
		i.cfg.OrchestratorURL+"/status",
		body,
	)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer TEST")

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to send health status to orchestrator")
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		logger.Warn().Int("status", resp.StatusCode).Str("url", req.URL.String()).Str("body", body.String()).Msg("Orchestrator returned non-200 status code")
		return err
	}

	return nil
}

func (i *IntercomService) SendJobHealth(health map[string]bool) {
	logger.Trace().Interface("health", health).Msg("Sending job health")
}
