package http_router

import (
	"encoding/json"
	"net/http"
	"time"

	"brume.dev/internal/common"
	middleware "brume.dev/internal/router/http/middleware"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/playground"
	"github.com/gorilla/mux"
	"github.com/rs/zerolog/log"
)

type LoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

func LoginHandler(w http.ResponseWriter, r *http.Request, authService *common.AuthentificationService) {
	body := r.Body
	defer body.Close()

	var loginRequest LoginRequest
	err := json.NewDecoder(body).Decode(&loginRequest)
	if err != nil {
		log.Error().Err(err).Msg("Failed to decode login request")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	token, err := authService.PasswordLogin(loginRequest.Email, loginRequest.Password)

	if err != nil {
		log.Error().Err(err).Msg("Failed to generate token")
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	log.Debug().Str("email", loginRequest.Email).Msg("User logged in")

	// set token in cookies
	http.SetCookie(w, &http.Cookie{
		Name:    "access_token",
		Value:   token,
		Expires: time.Now().Add(24 * time.Hour),
	})
	w.WriteHeader(http.StatusOK)
	w.Write([]byte(token))
}

func GeneralHTTPRouter(authService *common.AuthentificationService, public_gql *handler.Server) *mux.Router {
	router := mux.NewRouter()

	router.Path("/").HandlerFunc(playground.Handler("Brume GQL Playground", "/graphql"))
	router.Path("/health").HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("public healthy. yeah!"))
	})
	router.Path("/graphql").Handler(middleware.AuthMiddleware(public_gql))
	router.Path("/login").Methods(http.MethodPost).HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		LoginHandler(w, r, authService)
	})

	return router
}
