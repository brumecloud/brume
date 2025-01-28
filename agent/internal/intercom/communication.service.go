// package used to communicate with the orchestrator
package intercom_service

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"

	job_model "brume.dev/jobs/model"
	log_model "brume.dev/logs/model"
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
	jsonData, err := json.Marshal(map[string]interface{}{
		"job_id": job.ID,
		"bid":    bid,
	})
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal job data")
		return false, err
	}

	body := bytes.NewBuffer(jsonData)

	req, err := http.NewRequest("POST", i.cfg.OrchestratorURL+"/scheduler/v1/bid/"+job.ID.String(), body)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return false, err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+i.GenerateToken())
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
	req, err := http.NewRequest("GET", i.cfg.OrchestratorURL+"/scheduler/v1/job", nil)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+i.GenerateToken())
	req.Header.Set("X-Brume-Agent-ID", i.cfg.AgentID)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to send health status to orchestrator")
		return nil, err
	}

	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)

	logger.Info().Str("body", string(body)).Msg("Received jobs")

	if err != nil {
		logger.Warn().Err(err).Msg("Failed to read response body")
		return nil, err
	}

	var jobs []*job_model.Job
	err = json.Unmarshal(body, &jobs)
	if err != nil {
		logger.Warn().Err(err).Str("body", string(body)).Msg("Failed to unmarshal job")
		return nil, err
	}

	return jobs, nil
}

func (i *IntercomService) GetJobStatus(ctx context.Context, jobID string) (*job_model.JobStatus, error) {
	req, err := http.NewRequest("GET", i.cfg.OrchestratorURL+"/scheduler/v1/job/"+jobID, nil)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer TEST")
	req.Header.Set("X-Brume-Agent-ID", i.cfg.AgentID)

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

	jobStatus := &job_model.JobStatus{}
	err = json.Unmarshal(body, jobStatus)
	if err != nil {
		logger.Warn().Err(err).Str("body", string(body)).Msg("Failed to unmarshal job status")
		return nil, err
	}

	return jobStatus, nil
}

func (i *IntercomService) ReleaseJob(ctx context.Context, jobID string) error {
	req, err := http.NewRequest("POST", i.cfg.OrchestratorURL+"/scheduler/v1/release/"+jobID, nil)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+i.GenerateToken())
	req.Header.Set("X-Brume-Agent-ID", i.cfg.AgentID)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to send release job to orchestrator")
		return err
	}

	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		logger.Warn().Int("status", resp.StatusCode).Str("url", req.URL.String()).Msg("Orchestrator returned non-200 release job status code.")
		return err
	}

	return nil
}

func (i *IntercomService) SendRunningJobsHealth(jobHealth map[string]job_model.JobStatusEnum) error {
	// do HTTP call to the orchestrator
	jsonData, err := json.Marshal(map[string]interface{}{
		"machine_id": i.cfg.AgentID,
		"status":     jobHealth,
	})
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal health data")
		return err
	}

	body := bytes.NewBuffer(jsonData)

	req, err := http.NewRequest("POST", i.cfg.OrchestratorURL+"/monitoring/v1/jobs/status", body)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+i.GenerateToken())
	req.Header.Set("X-Brume-Agent-ID", i.cfg.AgentID)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to send running job health to orchestrator")
		return err
	}

	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		logger.Warn().Int("status", resp.StatusCode).Str("url", req.URL.String()).Str("body", body.String()).Msg("Orchestrator returned non-200 status code")
		return err
	}

	return nil
}

func (i *IntercomService) SendHealth(generalHealth string) error {
	// do HTTP call to the orchestrator
	jsonData, err := json.Marshal(map[string]interface{}{
		"machine_id": i.cfg.AgentID,
		"status":     generalHealth,
	})
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal health data")
		return err
	}

	body := bytes.NewBuffer(jsonData)

	req, err := http.NewRequest(
		"POST",
		i.cfg.OrchestratorURL+"/monitoring/v1/agent/status",
		body,
	)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+i.GenerateToken())

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

func (i *IntercomService) SendLogs(logs []*log_model.AgentLogs) {
	logs_request := map[string]interface{}{
		"logs":       logs,
		"machine_id": i.cfg.AgentID,
	}

	jsonData, err := json.Marshal(logs_request)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal logs data")
		return
	}

	body := bytes.NewBuffer(jsonData)

	req, err := http.NewRequest("POST", i.cfg.OrchestratorURL+"/monitoring/v1/jobs/logs", body)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to create request")
		return
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+i.GenerateToken())
	req.Header.Set("X-Brume-Agent-ID", i.cfg.AgentID)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to send logs to orchestrator")
		return
	}

	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		logger.Warn().Int("status", resp.StatusCode).Str("url", req.URL.String()).Str("body", body.String()).Msg("Orchestrator returned non-200 status code")
		return
	}
}

func (i *IntercomService) SendJobHealth(health map[string]bool) {
}

// this token is generated using the current time,
// sign using the agent private key
func (i *IntercomService) GenerateToken() string {
	return "TEST"
}
