package http_middleware

import (
	"context"
	"net/http"

	"brume.dev/internal/common"
	"brume.dev/internal/log"
)

var logger = log.GetLogger("http_middleware")

func AuthMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if len(r.Cookies()) == 0 {
			logger.Debug().Msg("Cookies not found")
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
			logger.Debug().Msg("No token found in cookies")
			w.WriteHeader(http.StatusUnauthorized)
			return
		}

		logger.Info().Str("token", token)
		claims, err := common.VerifyToken(token)
		if err != nil {
			w.WriteHeader(http.StatusForbidden)
			logger.Debug().Err(err).Msg("Failed to verify token")
			w.Write([]byte("Unauthorized"))
			return
		}

		ctx := context.WithValue(r.Context(), "user", claims.Subject)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}
