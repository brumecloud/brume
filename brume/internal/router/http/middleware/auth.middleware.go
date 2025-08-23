package http_middleware

import (
	"context"
	"net/http"

	"brume.dev/internal/log"
	brume_workos "brume.dev/internal/workos"
)

var logger = log.GetLogger("http_middleware")

func AuthMiddleware(workosClient *brume_workos.WorkOSClient, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if len(r.Cookies()) == 0 {
			logger.Warn().Msg("Cookies not found")
			w.WriteHeader(http.StatusUnauthorized)
			return
		}

		token := ""

		for _, cookie := range r.Cookies() {
			if cookie.Name == "access_token" {
				token = cookie.Value
				break
			}
		}

		if token == "" {
			logger.Warn().Msg("No token found in cookies")
			w.WriteHeader(http.StatusUnauthorized)
			return
		}

		logger.Info().Str("token", token)
		validatedToken, err := workosClient.VerifyToken(token)
		if err != nil {
			w.WriteHeader(http.StatusForbidden)
			logger.Warn().Err(err).Msg("Failed to verify token")
			w.Write([]byte("Unauthorized"))
			return
		}

		subject, ok := validatedToken.Subject()
		if !ok {
			logger.Panic().Msg("Failed to get subject from token AFTER verification")
		}

		ctx := context.WithValue(r.Context(), "user", subject)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}
