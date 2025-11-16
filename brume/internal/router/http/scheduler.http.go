package http_router

import (
	"encoding/json"
	"net/http"

	job_model "brume.dev/jobs/model"
	job_service "brume.dev/jobs/service"
	"github.com/gorilla/mux"
)

type SchedulerHTTPRouterV1 struct {
	bidService *job_service.BidService
	jobService *job_service.JobService
}

func NewSchedulerHTTPRouterV1(bidService *job_service.BidService, jobService *job_service.JobService) *SchedulerHTTPRouterV1 {
	return &SchedulerHTTPRouterV1{bidService: bidService, jobService: jobService}
}

func (s *SchedulerHTTPRouterV1) RegisterRoutes(router *mux.Router) {
	router.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("scheduler is alive. yeah!"))
	}).Methods(http.MethodGet)

	// ORCHESTRATOR -> AGENT
	// orchestrator core route
	// this route is used by agent to poll the scheduler for a job
	// their token is used to indentify them and get them the right job they
	// can run on their machine
	router.HandleFunc("/job", func(w http.ResponseWriter, r *http.Request) {
		// TODO: get only the right bids, not all
		machineID := r.Header.Get("X-Brume-Machine-ID")

		if machineID == "" {
			logger.Error().Msg("Machine ID is required")
			w.WriteHeader(http.StatusBadRequest)
			w.Write([]byte("Machine ID is required"))
			return
		}

		bids, err := s.bidService.GetAllCurrentBids()
		if err != nil {
			logger.Error().Err(err).Msg("Failed to get all current bids")
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte(err.Error()))
			return
		}

		logger.Trace().Str("machine_id", machineID).Int("bids", len(bids)).Msg("Sending bids to machine")
		json.NewEncoder(w).Encode(bids)
	}).Methods(http.MethodGet)

	// AGENT -> ORCHESTRATOR
	// orchestrator core route
	// multiple machine can bid for the same job, the scheduler will choose the best bid
	// once one bid is made, the scheduler waits 3s max before giving a response
	// TODO: for now the first bid is accepted
	router.HandleFunc("/bid/{bidId}", func(w http.ResponseWriter, r *http.Request) {
		machineID := r.Header.Get("X-Brume-Machine-ID")
		if machineID == "" {
			logger.Error().Str("bid_id", mux.Vars(r)["bidId"]).Msg("Machine id is required")
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		logger.Trace().Str("bid_id", mux.Vars(r)["bidId"]).Str("machine_id", machineID).Msg("Ingesting bid")

		bidID := mux.Vars(r)["bidId"]
		err := s.bidService.AcceptBid(bidID, machineID)
		if err != nil {
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte(err.Error()))
			return
		}

		w.WriteHeader(http.StatusOK)
		w.Write([]byte("OK"))
	}).Methods(http.MethodPost)

	// ORCHESTRATOR -> AGENT
	// edge route
	// this route is used to get the latest status of the job
	// it can be running or stoped by the orchestrator
	router.HandleFunc("/job/{jobId}", func(w http.ResponseWriter, r *http.Request) {
		jobID := mux.Vars(r)["jobId"]

		status, err := s.jobService.GetJobStatus(jobID)
		// this should never happen
		if err != nil {
			logger.Error().Err(err).Str("job_id", jobID).Msg("Failed to get job status")
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte(err.Error()))
			return
		}

		jobStatus := &job_model.JobStatus{
			JobID:  jobID,
			Status: status,
		}

		json.NewEncoder(w).Encode(jobStatus)
	}).Methods(http.MethodGet)

	// AGENT -> ORCHESTRATOR
	// orchestrator core route
	// it is used to inform the orchestrator that the job "released"
	// a release job is failed or stopped
	router.HandleFunc("/release/{jobId}", func(w http.ResponseWriter, r *http.Request) {
		logger.Warn().Str("job_id", mux.Vars(r)["jobId"]).Msg("Releasing job is not implemented yet")

		// logger.Trace().Str("job_id", mux.Vars(r)["jobId"]).Msg("Releasing job")
		w.WriteHeader(http.StatusNotImplemented)
	}).Methods(http.MethodPost)
}
