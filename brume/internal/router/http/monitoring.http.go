package http_router

import (
	"encoding/json"
	"net/http"

	job_model "brume.dev/jobs/model"
	job_service "brume.dev/jobs/service"
	log "brume.dev/logs"
	log_model "brume.dev/logs/model"
	"brume.dev/machine"
	"github.com/go-playground/validator/v10"
	"github.com/google/uuid"
	"github.com/gorilla/mux"
)

type MonitoringHTTPRouterV1 struct {
	machineService *machine.MachineService
	jobService     *job_service.JobService
	logService     *log.LogService
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
	MachineId uuid.UUID              `json:"machine_id" validate:"required"`
	Logs      []*log_model.AgentLogs `json:"logs" validate:"required,dive"`
}

var Validator = validator.New()

func NewMonitoringHTTPRouterV1(machineService *machine.MachineService, jobService *job_service.JobService, logService *log.LogService) *MonitoringHTTPRouterV1 {
	return &MonitoringHTTPRouterV1{
		machineService: machineService,
		jobService:     jobService,
		logService:     logService,
	}
}

func (m *MonitoringHTTPRouterV1) RegisterRoutes(router *mux.Router) {
	router.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("monitoring is alive. yeah!"))
	}).Methods(http.MethodGet)

	// AGENT -> ORCHESTRATOR
	// edge route
	// send the general health of the agent
	router.HandleFunc("/agent/status", func(w http.ResponseWriter, r *http.Request) {
		// logger.Trace().Msg("Ingesting agent status")

		var req StatusRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			logger.Error().Err(err).Msg("Could not decode request")
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		val := validator.New(validator.WithRequiredStructEnabled())
		if err := val.Struct(req); err != nil {
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

	// AGENT -> ORCHESTRATOR
	// edge route
	// send the status of the running jobs
	router.HandleFunc("/jobs/status", func(w http.ResponseWriter, r *http.Request) {
		var req JobsStatusRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		val := validator.New(validator.WithRequiredStructEnabled())
		if err := val.Struct(req); err != nil {
			logger.Error().Err(err).Msg("Could not validate request")
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		var err error

		// dont stop the request
		go func() {
			for jobID, status := range req.Status {
				err = m.jobService.SetJobHealth(jobID)
				if err != nil {
					// dont stop trying to record the status
					// we should have a queue to retry
					// TODO: create a queue to retry the job status
					logger.Error().Err(err).Str("job_id", jobID).Str("status", string(status)).Msg("Failed to record job status")
				}
			}
		}()

		w.WriteHeader(http.StatusOK)
	}).Methods(http.MethodPost)

	// AGENT -> ORCHESTRATOR
	// edge route
	// this will ingest all the log from the agent
	router.HandleFunc("/jobs/logs", func(w http.ResponseWriter, r *http.Request) {
		var req LogsRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		val := validator.New(validator.WithRequiredStructEnabled())
		if err := val.Struct(req); err != nil {
			logger.Error().Err(err).Msg("Could not validate request")
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		logger.Debug().Interface("logs", req.Logs).Msg("logs in http")

		// this is calling directly the clickhouse database
		err := m.logService.IngestLogs(req.Logs)
		if err != nil {
			logger.Error().Err(err).Msg("Failed to ingest logs")
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		w.WriteHeader(http.StatusOK)
	}).Methods(http.MethodPost)
}
