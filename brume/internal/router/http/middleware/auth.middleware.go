package http_middleware

import (
	"context"
	"net/http"

	"brume.dev/internal/log"
	brume_workos "brume.dev/internal/workos"
)

var logger = log.GetLogger("router.http.middleware")

func AuthMiddleware(workosClient *brume_workos.WorkOSClient, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if len(r.Cookies()) == 0 {
			logger.Warn().Msg("Cookies not found")
			w.WriteHeader(http.StatusUnauthorized)
			return
		}

		access_token := ""
		refresh_token := ""

		for _, cookie := range r.Cookies() {
			if cookie.Name == "access_token" {
				access_token = cookie.Value
				continue
			}
			if cookie.Name == "refresh_token" {
				refresh_token = cookie.Value
				continue
			}
		}

		if access_token == "" {
			logger.Warn().Msg("No token found in cookies")
			w.WriteHeader(http.StatusUnauthorized)
			return
		}

		validatedToken, err := workosClient.VerifyTokenWithRefresh(access_token, refresh_token, w)
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

		// set the provider id in the context
		ctx := context.WithValue(r.Context(), "user", subject)

		next.ServeHTTP(w, r.WithContext(ctx))
	})
}
