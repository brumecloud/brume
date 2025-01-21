package http_router

import (
	"encoding/json"
	"net/http"

	job_model "brume.dev/jobs/model"
	job_service "brume.dev/jobs/service"
	"brume.dev/machine"
	"github.com/go-playground/validator/v10"
	"github.com/google/uuid"
	"github.com/gorilla/mux"
)

type MonitoringHTTPRouterV1 struct {
	machineService *machine.MachineService
	jobService     *job_service.JobService
}

type StatusRequest struct {
	// machine id will comes from the token later
	MachineId uuid.UUID `json:"machine_id" validate:"required"`
	Status    string    `json:"status" validate:"required"`
}

type JobsStatusRequest struct {
	MachineId uuid.UUID                          `json:"machine_id" validate:"required"`
	Status    map[string]job_model.JobStatusEnum `json:"status" validate:"required"`
}

type LogsRequest struct {
	MachineId uuid.UUID `json:"machine_id" validate:"required"`
	Logs      []string  `json:"logs" validate:"required"`
}

var Validator = validator.New()

func NewMonitoringHTTPRouterV1(machineService *machine.MachineService, jobService *job_service.JobService) *MonitoringHTTPRouterV1 {
	return &MonitoringHTTPRouterV1{
		machineService: machineService,
		jobService:     jobService,
	}
}

func (m *MonitoringHTTPRouterV1) RegisterRoutes(router *mux.Router) {
	router.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("monitoring is alive. yeah!"))
	}).Methods(http.MethodGet)

	// send the general health of the agent
	router.HandleFunc("/agent/status", func(w http.ResponseWriter, r *http.Request) {
		// logger.Trace().Msg("Ingesting agent status")

		var req StatusRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			logger.Error().Err(err).Msg("Could not decode request")
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		if err := Validator.Struct(req); err != nil {
			logger.Error().Err(err).Msg("Could not validate request")
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		// dont stop the request
		go func() {
			err := m.machineService.RecordStatus(req.MachineId, req.Status)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
		}()

		w.WriteHeader(http.StatusOK)
	}).Methods(http.MethodPost)

	// send the status of the running jobs
	router.HandleFunc("/jobs/status", func(w http.ResponseWriter, r *http.Request) {
		var req JobsStatusRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		if err := Validator.Struct(req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		var err error

		// dont stop the request
		go func() {
			for jobID, status := range req.Status {
				err = m.jobService.RecordJobStatus(jobID, status)
				if err != nil {
					// dont stop trying to record the status
					// we should have a queue to retry
					// TODO: create a queue to retry the job status
					logger.Error().Err(err).Str("job_id", jobID).Str("status", string(status)).Msg("Failed to record job status")
				}
			}
		}()

		w.WriteHeader(http.StatusAccepted)
	}).Methods(http.MethodPost)

	// this will ingest all the log from the agent
	router.HandleFunc("/jobs/logs", func(w http.ResponseWriter, r *http.Request) {
		logger.Trace().Msg("Ingesting agent logs")

		var req LogsRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		if err := Validator.Struct(req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		// TODO: ingest logs to clickhouse
		// in batch

		w.WriteHeader(http.StatusOK)
		w.Write([]byte("OK"))
	}).Methods(http.MethodPost)
}
