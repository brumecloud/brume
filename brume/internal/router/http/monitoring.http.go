package http_router

import (
	"encoding/json"
	"net/http"

	"brume.dev/machine"
	"github.com/go-playground/validator/v10"
	"github.com/google/uuid"
	"github.com/gorilla/mux"
)

type MonitoringHTTPRouterV1 struct {
	machineService *machine.MachineService
}

type StatusRequest struct {
	// machine id will comes from the token later
	MachineId uuid.UUID `json:"machine_id" validate:"required"`
	Status    string    `json:"status" validate:"required"`
}

type LogsRequest struct {
	MachineId uuid.UUID `json:"machine_id" validate:"required"`
	Logs      []string  `json:"logs" validate:"required"`
}

var Validator = validator.New()

func NewMonitoringHTTPRouterV1(machineService *machine.MachineService) *MonitoringHTTPRouterV1 {
	return &MonitoringHTTPRouterV1{
		machineService: machineService,
	}
}

func (m *MonitoringHTTPRouterV1) RegisterRoutes(router *mux.Router) {
	router.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("monitoring is alive. yeah!"))
	}).Methods(http.MethodGet)

	router.HandleFunc("/status", func(w http.ResponseWriter, r *http.Request) {
		logger.Trace().Msg("Ingesting agent status")

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

		err := m.machineService.RecordStatus(req.MachineId, req.Status)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		w.WriteHeader(http.StatusOK)
		w.Write([]byte("OK"))
	}).Methods(http.MethodPost)

	router.HandleFunc("/logs", func(w http.ResponseWriter, r *http.Request) {
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
