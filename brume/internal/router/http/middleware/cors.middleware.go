package http_middleware

import "github.com/rs/cors"

var CorsHandler = cors.New(cors.Options{
	AllowedOrigins:   []string{"http://localhost:5173", "http://localhost:80", "http://localhost:3000"},
	AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
	AllowedHeaders:   []string{"Accept", "Authorization", "Content-Type"},
	AllowCredentials: true,
})
