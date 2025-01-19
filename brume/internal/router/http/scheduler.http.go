package http_router

import (
	"encoding/json"
	"net/http"

	job_service "brume.dev/jobs/service"
	"github.com/google/uuid"
	"github.com/gorilla/mux"
)

type SchedulerHTTPRouterV1 struct {
	bidService *job_service.BidService
}

func NewSchedulerHTTPRouterV1(bidService *job_service.BidService) *SchedulerHTTPRouterV1 {
	return &SchedulerHTTPRouterV1{bidService: bidService}
}

func (s *SchedulerHTTPRouterV1) RegisterRoutes(router *mux.Router) {
	router.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("scheduler is alive. yeah!"))
	}).Methods(http.MethodGet)

	// AGENT -> ORCHESTRATOR
	// this route is used by agent to poll the scheduler for a job
	// their token is used to indentify them and get them the right job they
	// can run on their machine
	router.HandleFunc("/job", func(w http.ResponseWriter, r *http.Request) {
		// TODO: get only the right bids, not all
		agentID := r.Header.Get("X-Brume-Agent-ID")

		if agentID == "" {
			logger.Error().Msg("Agent ID is required")
			w.WriteHeader(http.StatusBadRequest)
			w.Write([]byte("Agent ID is required"))
			return
		}

		bids, err := s.bidService.GetAllCurrentBids()
		if err != nil {
			logger.Error().Err(err).Msg("Failed to get all current bids")
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte(err.Error()))
			return
		}

		logger.Info().Str("agent_id", agentID).Int("bids", len(bids)).Msg("Sending bids to agent")
		json.NewEncoder(w).Encode(bids)
	}).Methods(http.MethodGet)

	// AGENT -> ORCHESTRATOR
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

		logger.Info().Str("bid_id", mux.Vars(r)["bidId"]).Str("machine_id", machineID).Msg("Ingesting bid")

		machineIDUUID, err := uuid.Parse(machineID)
		if err != nil {
			logger.Error().Err(err).Str("bid_id", mux.Vars(r)["bidId"]).Msg("Failed to parse machine id")
			w.WriteHeader(http.StatusBadRequest)
			w.Write([]byte(err.Error()))
			return
		}

		bidID := mux.Vars(r)["bidId"]
		err = s.bidService.AcceptBid(bidID, machineIDUUID)
		if err != nil {
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte(err.Error()))
			return
		}

		w.WriteHeader(http.StatusOK)
		w.Write([]byte("OK"))
	}).Methods(http.MethodPost)

	// ORCHESTRATOR -> AGENT
	// this route is used to get the latest status of the job
	// it can be running or stoped by the orchestrator
	router.HandleFunc("/job/{jobId}", func(w http.ResponseWriter, r *http.Request) {
		logger.Warn().Str("job_id", mux.Vars(r)["jobId"]).Msg("Getting the job status is not implemented yet")

		// logger.Trace().Str("job_id", mux.Vars(r)["jobId"]).Msg("Getting job")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("job"))
	}).Methods(http.MethodGet)

	// this is a AGENT -> ORCHESTRATOR route
	// it is used to inform the orchestrator that the job "released"
	// a release job is failed or stopped
	router.HandleFunc("/release/{jobId}", func(w http.ResponseWriter, r *http.Request) {
		logger.Warn().Str("job_id", mux.Vars(r)["jobId"]).Msg("Releasing job is not implemented yet")

		// logger.Trace().Str("job_id", mux.Vars(r)["jobId"]).Msg("Releasing job")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("release"))
	}).Methods(http.MethodPost)
}
