package http_router

import (
	"net/http"

	"github.com/gorilla/mux"
)

type MonitoringHTTPRouterV1 struct{}

func NewMonitoringHTTPRouterV1() *MonitoringHTTPRouterV1 {
	return &MonitoringHTTPRouterV1{}
}

func (m *MonitoringHTTPRouterV1) RegisterRoutes(router *mux.Router) {
	router.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("monitoring is alive. yeah!"))
	}).Methods(http.MethodGet)

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
}
