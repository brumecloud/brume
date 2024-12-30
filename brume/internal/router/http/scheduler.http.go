package http_router

import (
	"encoding/json"
	"net/http"

	job_service "brume.dev/internal/jobs/service"
	"github.com/gorilla/mux"
	"github.com/rs/zerolog/log"
)

var logger = log.With().Str("module", "http").Logger()

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

	// this route is used by agent to poll the scheduler for a job
	// their token is used to indentify them and get them the right job they
	// can run on their machine
	router.HandleFunc("/job", func(w http.ResponseWriter, r *http.Request) {
		// TODO: get only the right bids, not all
		bids, err := s.bidService.GetAllCurrentBids()
		if err != nil {
			logger.Error().Err(err).Msg("Failed to get all current bids")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		json.NewEncoder(w).Encode(bids)
		w.WriteHeader(http.StatusOK)
	}).Methods(http.MethodGet)

	// multiple machine can bid for the same job, the scheduler will choose the best bid
	// once one bid is made, the scheduler waits 3s max before giving a response
	router.HandleFunc("/bid/{jobId}", func(w http.ResponseWriter, r *http.Request) {
		logger.Trace().Msg("Getting bid")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("bid"))
	}).Methods(http.MethodPost)

	// once the job is finished (failed or not) on the machine side, the agent will send a release request
	// this is used to inform the scheduler that the job is done and the machine is free to bid for a new job
	router.HandleFunc("/release/{jobId}", func(w http.ResponseWriter, r *http.Request) {
		logger.Trace().Msg("Releasing agent")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("release"))
	}).Methods(http.MethodPost)
}
