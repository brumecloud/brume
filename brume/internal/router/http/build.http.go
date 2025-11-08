package http_router

import (
	"net/http"

	"github.com/gorilla/mux"
)

// /v1/build
// return the list of build an agent might consider
func GetBuildHandler(w http.ResponseWriter, r *http.Request) {
}

// /v1/build
// update a build (starting, finished, failed)
func PostBuildHandler(w http.ResponseWriter, r *http.Request) {
}

func BuildHTTPRouter() {
	router := mux.NewRouter()

	router.Handle("/v1/build", http.HandlerFunc(GetBuildHandler)).Methods(http.MethodGet)
	router.Handle("/v1/build", http.HandlerFunc(PostBuildHandler)).Methods(http.MethodPost)
}