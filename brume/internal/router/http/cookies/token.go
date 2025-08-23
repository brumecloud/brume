package cookies

import (
	"net/http"
	"time"

	"brume.dev/internal/config"
)

func GenerateUserCookie(accessToken string, refreshToken string, cfg *config.BrumeConfig, w http.ResponseWriter) *http.Cookie {
	http.SetCookie(w, &http.Cookie{
		Name:     "access_token",
		Value:    accessToken,
		Path:     "/",
		Expires:  time.Now().Add(4 * time.Hour),
		HttpOnly: true,
		Secure:   false,
		SameSite: http.SameSiteLaxMode,
	})
	http.SetCookie(w, &http.Cookie{
		Name:     "refresh_token",
		Value:    refreshToken,
		Path:     "/",
		HttpOnly: true,
		Secure:   false,
		SameSite: http.SameSiteLaxMode,
	})
	return nil
}
