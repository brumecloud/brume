package http_router

import (
	"context"
	"encoding/json"
	"net/http"
	"time"

	"brume.dev/internal/common"
	config "brume.dev/internal/config"
	brume_log "brume.dev/internal/log"
	middleware "brume.dev/internal/router/http/middleware"
	brume_workos "brume.dev/internal/workos"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/playground"
	"github.com/gorilla/mux"
	"github.com/workos/workos-go/v4/pkg/usermanagement"
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
		logger.Error().Err(err).Msg("Failed to decode login request")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	token, err := authService.PasswordLogin(loginRequest.Email, loginRequest.Password)
	if err != nil {
		logger.Error().Err(err).Msg("Failed to generate token")
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	logger.Debug().Str("email", loginRequest.Email).Msg("User logged in")

	// set token in cookies
	http.SetCookie(w, &http.Cookie{
		Name:    "access_token",
		Value:   token,
		Expires: time.Now().Add(24 * time.Hour),
	})
	w.WriteHeader(http.StatusOK)
	w.Write([]byte(token))
}

var workosLogger = brume_log.GetLogger("workos")

func GeneralHTTPRouter(authService *common.AuthentificationService, public_gql *handler.Server, workosClient *brume_workos.WorkOSClient, cfg *config.BrumeConfig) *mux.Router {
	router := mux.NewRouter()
	usermanagement.SetAPIKey(cfg.WorkOSConfig.ClientSecret)

	router.Handle("/", playground.Handler("Brume GQL Playground", "/graphql")).Methods(http.MethodGet)

	router.Handle("/health", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("public healthy. yeah!"))
	})).Methods(http.MethodGet)

	router.Handle("/graphql", middleware.AuthMiddleware(workosClient, public_gql)).Methods(http.MethodPost)

	router.Handle("/login", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		LoginHandler(w, r, authService)
	})).Methods(http.MethodPost)

	// start the magic auth flow
	router.Handle("/wos/magic-link", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		email := r.URL.Query().Get("email")

		_, err := usermanagement.CreateMagicAuth(context.Background(), usermanagement.CreateMagicAuthOpts{
			Email: email,
		})
		if err != nil {
			logger.Error().Err(err).Msg("Failed to send the magic code")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		w.WriteHeader(http.StatusOK)
	})).Methods(http.MethodGet)

	// start the magic auth flow
	router.Handle("/wos/magic-code", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		code := r.URL.Query().Get("code")
		email := r.URL.Query().Get("email")

		user, err := usermanagement.AuthenticateWithMagicAuth(context.Background(), usermanagement.AuthenticateWithMagicAuthOpts{
			Code:     code,
			ClientID: cfg.WorkOSConfig.ClientID,
			Email:    email,
		})
		if err != nil {
			logger.Error().Err(err).Msg("Failed to send the magic code")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		generateUserCookie(&user, cfg, w)

		userJson, err := json.Marshal(user)
		if err != nil {
			workosLogger.Error().Err(err).Msg("Failed to marshal user")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		w.Write(userJson)
	})).Methods(http.MethodGet)

	// authenticate with password
	router.Handle("/wos/pass", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		workosLogger.Info().Msg("Authenticating with password")
		body := r.Body
		defer body.Close()

		var loginRequest LoginRequest
		err := json.NewDecoder(body).Decode(&loginRequest)
		if err != nil {
			workosLogger.Error().Err(err).Msg("Failed to decode login request")
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		user, err := usermanagement.AuthenticateWithPassword(context.Background(), usermanagement.AuthenticateWithPasswordOpts{
			ClientID: cfg.WorkOSConfig.ClientID,
			Email:    loginRequest.Email,
			Password: loginRequest.Password,
		})
		if err != nil {
			workosLogger.Error().Err(err).Msg("Failed to authenticate with password")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		http.SetCookie(w, &http.Cookie{
			Name:     "access_token",
			Value:    user.AccessToken,
			Expires:  time.Now().Add(4 * time.Hour),
			Secure:   !cfg.BrumeGeneralConfig.IsDev,
			HttpOnly: true,
			SameSite: http.SameSiteNoneMode,
		})

		http.SetCookie(w, &http.Cookie{
			Name:     "refresh_token",
			Value:    user.RefreshToken,
			Secure:   !cfg.BrumeGeneralConfig.IsDev,
			HttpOnly: true,
			SameSite: http.SameSiteNoneMode,
		})

		userJson, err := json.Marshal(user)
		if err != nil {
			workosLogger.Error().Err(err).Msg("Failed to marshal user")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		w.Write(userJson)
		w.WriteHeader(http.StatusOK)
	})).Methods(http.MethodPost)

	// start the oauth flow
	router.Handle("/wos/oauth", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		workosLogger.Info().Msg("Getting authorization URL")
		url, err := usermanagement.GetAuthorizationURL(usermanagement.GetAuthorizationURLOpts{
			ClientID:    cfg.WorkOSConfig.ClientID,
			RedirectURI: cfg.WorkOSConfig.RedirectURI,
		})
		if err != nil {
			workosLogger.Error().Err(err).Msg("Failed to get authorization URL")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		workosLogger.Info().Str("url", url.String()).Msg("Redirecting to authorization URL")

		http.Redirect(w, r, url.String(), http.StatusFound)
	})).Methods(http.MethodGet)

	// callback after oauth flow
	router.Handle("/wos/callback", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		workosLogger.Info().Msg("Callback after oauth flow")
		code := r.URL.Query().Get("code")

		if code == "" {
			workosLogger.Error().Msg("WorkOS callback without code")
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		user, err := usermanagement.AuthenticateWithMagicAuth(context.Background(), usermanagement.AuthenticateWithMagicAuthOpts{
			Code:     code,
			ClientID: cfg.WorkOSConfig.ClientID,
		})
		if err != nil {
			workosLogger.Error().Err(err).Msg("Failed to get profile")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		http.SetCookie(w, &http.Cookie{
			Name:     "access_token",
			Value:    user.AccessToken,
			Expires:  time.Now().Add(4 * time.Hour),
			Secure:   !cfg.BrumeGeneralConfig.IsDev,
			HttpOnly: true,
			SameSite: http.SameSiteNoneMode,
		})

		http.SetCookie(w, &http.Cookie{
			Name:     "refresh_token",
			Value:    user.RefreshToken,
			Secure:   !cfg.BrumeGeneralConfig.IsDev,
			HttpOnly: true,
			SameSite: http.SameSiteNoneMode,
		})

		// TODO: Implement user management
		w.WriteHeader(http.StatusOK)

		userJson, err := json.Marshal(user)
		if err != nil {
			workosLogger.Error().Err(err).Msg("Failed to marshal user")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		w.Write(userJson)
		w.WriteHeader(http.StatusOK)
	})).Methods(http.MethodGet)

	return router
}

func generateUserCookie(user *usermanagement.AuthenticateResponse, cfg *config.BrumeConfig, w http.ResponseWriter) *http.Cookie {
	http.SetCookie(w, &http.Cookie{
		Name:     "access_token",
		Value:    user.AccessToken,
		Path:     "/",
		Expires:  time.Now().Add(4 * time.Hour),
		HttpOnly: true,
		Secure:   false,
		SameSite: http.SameSiteLaxMode,
	})
	http.SetCookie(w, &http.Cookie{
		Name:     "refresh_token",
		Value:    user.RefreshToken,
		Path:     "/",
		HttpOnly: true,
		Secure:   false,
		SameSite: http.SameSiteLaxMode,
	})
	return nil
}
