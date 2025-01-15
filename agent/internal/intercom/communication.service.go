// package used to communicate with the orchestrator
package intercom_service

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"

	job_model "brume.dev/jobs/model"
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

func (i *IntercomService) PlaceBid(ctx context.Context, job *job_model.Job, bid int) (bool, error) {
	logger.Trace().Msg("Placing bid")

	jsonData, err := json.Marshal(map[string]interface{}{
		"job_id": job.ID,
		"bid":    bid,
	})
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal job data")
		return false, err
	}

	body := bytes.NewBuffer(jsonData)

	req, err := http.NewRequest("POST", i.cfg.OrchestratorURL+"/bid/"+job.ID.String(), body)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return false, err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer TEST")
	req.Header.Set("X-Brume-Machine-ID", i.cfg.AgentID)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to send bid to orchestrator")
		return false, err
	}

	defer resp.Body.Close()

	if resp.StatusCode >= http.StatusBadRequest {
		logger.Warn().Int("status", resp.StatusCode).Str("url", req.URL.String()).Str("body", body.String()).Msg("Orchestrator returned non-200 status code")
		return false, err
	}

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to read response body")
		return false, err
	}

	// this will start the runner on the machine
	if string(respBody) == "OK" {
		logger.Info().Str("job_id", job.ID.String()).Msg("Bid accepted successfully")
		return true, nil
	}

	logger.Warn().Str("job_id", job.ID.String()).Msg("Bid not accepted")
	return false, nil
}

func (i *IntercomService) GetJobs(ctx context.Context) ([]*job_model.Job, error) {
	logger.Trace().Msg("Getting job")

	req, err := http.NewRequest("GET", i.cfg.OrchestratorURL+"/job", nil)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer TEST")

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to send health status to orchestrator")
		return nil, err
	}

	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to read response body")
		return nil, err
	}

	var jobs []*job_model.Job
	err = json.Unmarshal(body, &jobs)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to unmarshal job")
		return nil, err
	}

	return jobs, nil
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
		i.cfg.OrchestratorURL+"/monitoring/v1/status",
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
