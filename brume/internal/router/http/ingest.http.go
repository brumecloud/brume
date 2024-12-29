package http_router

import (
	"net/http"

	"github.com/gorilla/mux"
	"github.com/rs/zerolog/log"
)

var logger = log.With().Str("module", "http").Logger()

func AgentHTTPRouterV1() *mux.Router {
	base := mux.NewRouter()
	router := base.PathPrefix("/v1").Subrouter()

	router.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("ingest healthy. yeah!"))
	}).Methods(http.MethodGet)

	router.HandleFunc("/job", func(w http.ResponseWriter, r *http.Request) {
		logger.Trace().Msg("Sending job")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("job"))
	}).Methods(http.MethodGet)

	router.HandleFunc("/bid", func(w http.ResponseWriter, r *http.Request) {
		logger.Trace().Msg("Getting bid")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("bid"))
	}).Methods(http.MethodPost)

	router.HandleFunc("/status", func(w http.ResponseWriter, r *http.Request) {
		logger.Trace().Msg("Ingesting agent status")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("status"))
	}).Methods(http.MethodPost)

	router.HandleFunc("/logs", func(w http.ResponseWriter, r *http.Request) {
		logger.Trace().Msg("Ingesting agent logs")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("logs"))
	}).Methods(http.MethodPost)

	return router
}
