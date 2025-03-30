// package used to communicate with the orchestrator
package intercom_service

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"

	job_model "brume.dev/jobs/model"
	"github.com/brumecloud/agent/internal/config"
	"github.com/brumecloud/agent/internal/log"
)

var logger = log.GetLogger("intercom")

type IntercomService struct {
	cfg *config.GeneralConfig
}

func NewIntercomService(cfg *config.GeneralConfig) *IntercomService {
	return &IntercomService{
		cfg: cfg,
	}
}

func (i *IntercomService) PlaceBid(ctx context.Context, job *job_model.Job, bid int) (bool, error) {
	logger.Info().Str("job_id", job.ID.String()).Msg("Placing bid to orchestrator")
	jsonData, err := json.Marshal(map[string]interface{}{
		"job_id": job.ID,
		"bid":    bid,
	})
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal job data")
		return false, err
	}

	respBody, err := i.sendRequest("POST", i.cfg.Orchestrator.URL+"/scheduler/v1/bid/"+job.ID.String(), jsonData)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to send bid to orchestrator")
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
	logger.Info().Msg("Getting jobs from orchestrator")
	body, err := i.sendRequest("GET", i.cfg.Orchestrator.URL+"/scheduler/v1/job", nil)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to get jobs")
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

// Get the job status from the orchestrator
// this is used to see if the job should be stopped
func (i *IntercomService) GetJobStatus(ctx context.Context, jobID string) (*job_model.JobStatus, error) {
	logger.Info().Str("job_id", jobID).Msg("Getting job status to orchestrator")
	body, err := i.sendRequest("GET", i.cfg.Orchestrator.URL+"/scheduler/v1/job/"+jobID, nil)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to get job status")
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

// Release a job from the agent
// Send a request to the orchestrator informing that the job is no longer running on the machine
func (i *IntercomService) ReleaseJob(ctx context.Context, jobID string) error {
	_, err := i.sendRequest("POST", i.cfg.Orchestrator.URL+"/scheduler/v1/release/"+jobID, nil)
	return err
}

// Send the job metadata to the orchestrator
func (i *IntercomService) SendJobMetadata(ctx context.Context, jobID string, metadata job_model.JobMetadata) error {
	logger.Info().Str("job_id", jobID).Msg("Sending job metadata to orchestrator")
	jsonData, err := json.Marshal(metadata)
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal job status")
		return err
	}

	_, err = i.sendRequest("POST", i.cfg.Orchestrator.URL+"/monitoring/v1/jobs/"+jobID+"/metadata", jsonData)
	return err
}

// Send in batch the health of all the jobs running on the machine
func (i *IntercomService) SendJobsHealth(jobHealth map[string]job_model.JobStatus) error {
	// do HTTP call to the orchestrator
	jsonData, err := json.Marshal(map[string]interface{}{
		"machine_id": i.cfg.MachineID,
		"status":     jobHealth,
	})
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal health data")
		return err
	}

	_, err = i.sendRequest("POST", i.cfg.Orchestrator.URL+"/monitoring/v1/jobs/status", jsonData)
	return err
}

// Send the health of the machine to the orchestrator
func (i *IntercomService) SendMachineHealth(generalHealth string) error {
	// do HTTP call to the orchestrator
	jsonData, err := json.Marshal(map[string]interface{}{
		"machine_id": i.cfg.MachineID,
		"status":     generalHealth,
	})
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to marshal health data")
		return err
	}

	_, err = i.sendRequest("POST", i.cfg.Orchestrator.URL+"/monitoring/v1/agent/status", jsonData)
	return err
}

// this token is generated using the current time,
// sign using the agent private key
func (i *IntercomService) GenerateToken() string {
	return "TEST"
}

// private function to send a request to the orchestrator
func (i *IntercomService) sendRequest(method, url string, body []byte) ([]byte, error) {
	req, err := http.NewRequest(method, url, bytes.NewBuffer(body))
	if err != nil {
		return nil, err
	}

	logger.Info().Str("method", method).Str("url", url).Msg("Sending request")

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+i.GenerateToken())
	req.Header.Set("X-Brume-Machine-ID", i.cfg.MachineID)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	defer resp.Body.Close()

	body, err = io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		logger.Warn().Int("status", resp.StatusCode).Str("url", req.URL.String()).Msg("Orchestrator returned non-200 status code")
		return nil, errors.New(string(body))
	}

	return body, nil
}
