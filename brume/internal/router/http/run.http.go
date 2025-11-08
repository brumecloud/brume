package http_router

import (
	"net/http"

	"github.com/gorilla/mux"
)

// /v1/run
// return the list of run an agent might consider
func GetRunHandler(w http.ResponseWriter, r *http.Request) {
}

// /v1/run
// update a run (starting, finished, failed)
func PostRunHandler(w http.ResponseWriter, r *http.Request) {
}

func RunHTTPRouter() {
	router := mux.NewRouter()

	router.Handle("/v1/run", http.HandlerFunc(GetRunHandler)).Methods(http.MethodGet)
	router.Handle("/v1/run", http.HandlerFunc(PostRunHandler)).Methods(http.MethodPost)
}
