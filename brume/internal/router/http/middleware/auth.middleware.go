package http_middleware

import (
	"context"
	"net/http"

	"brume.dev/internal/common"
	"github.com/rs/zerolog/log"
)

func AuthMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if len(r.Cookies()) == 0 {
			log.Debug().Msg("Cookies not found")
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
			log.Debug().Msg("No token found in cookies")
			w.WriteHeader(http.StatusUnauthorized)
			return
		}

		log.Info().Str("token", token)
		claims, err := common.VerifyToken(token)

		if err != nil {
			w.WriteHeader(http.StatusForbidden)
			log.Debug().Err(err).Msg("Failed to verify token")
			w.Write([]byte("Unauthorized"))
			return
		}

		ctx := context.WithValue(r.Context(), "user", claims.Subject)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}
