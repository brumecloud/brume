// package used to communicate with the orchestrator
package intercom_service

import (
	"bytes"
	"encoding/json"
	"net/http"
	"time"

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

func (i *IntercomService) SendGeneralHealth(health bool) {
	logger.Trace().Bool("health", health).Msg("Sending general health")

	// do HTTP call to the orchestrator
	jsonData, err := json.Marshal(map[string]interface{}{
		"health":    true,
		"agent_id":  i.cfg.AgentID,
		"timestamp": time.Now().Unix(),
	})

	if err != nil {
		logger.Error().Err(err).Msg("Failed to marshal health data")
		return
	}

	req, err := http.NewRequest(
		"POST",
		i.cfg.OrchestratorURL+"/api/v1/agent/health",
		bytes.NewBuffer(jsonData),
	)

	if err != nil {
		logger.Error().Err(err).Msg("Failed to create request")
		return
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer TEST")

	client := &http.Client{}
	resp, err := client.Do(req)

	if err != nil {
		logger.Error().Err(err).Msg("Failed to send health status to orchestrator")
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		logger.Error().Int("status", resp.StatusCode).Msg("Orchestrator returned non-200 status code")
		return
	}
}

func (i *IntercomService) SendJobHealth(health map[string]bool) {
	logger.Trace().Interface("health", health).Msg("Sending job health")
}
