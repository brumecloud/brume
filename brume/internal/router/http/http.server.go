package http_router

import (
	"context"
	"encoding/json"
	"net/http"
	"time"

	"brume.dev/account/user"
	"brume.dev/internal/common"
	public_graph "brume.dev/internal/router/public-gql/graph"
	public_graph_generated "brume.dev/internal/router/public-gql/graph/generated/generated.go"
	brume_log "brume.dev/logs"
	"brume.dev/project"
	"brume.dev/service"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/handler/transport"
	"github.com/99designs/gqlgen/graphql/playground"
	"github.com/rs/cors"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type BrumeHTTPServer struct{}

type LoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

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

func NewHTTPServer(lc fx.Lifecycle, authentificationService *common.AuthentificationService, userService *user.UserService, projectService *project.ProjectService, serviceService *service.ServiceService, logService *brume_log.LogService) *BrumeHTTPServer {
	log.Info().Msg("Launching the HTTP Server")

	cors := cors.New(cors.Options{
		AllowedOrigins:   []string{"http://localhost:5173"},
		AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders:   []string{"*"},
		AllowCredentials: true,
		MaxAge:           300,
	})

	public_resolver := &public_graph.Resolver{
		UserService:    userService,
		ProjectService: projectService,
		ServiceService: serviceService,
		LogService:     logService,
	}

	public_gql := handler.NewDefaultServer(public_graph_generated.NewExecutableSchema(public_graph_generated.Config{Resolvers: public_resolver}))
	public_gql.AddTransport(&transport.Websocket{})

	http.Handle("/", cors.Handler(playground.Handler("Brume GQL Playground", "/graphql")))
	http.Handle("/graphql", cors.Handler(AuthMiddleware(public_gql)))

	http.Handle("/login", cors.Handler(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method == "POST" {
			println("login request")
			body := r.Body
			defer body.Close()

			var loginRequest LoginRequest
			err := json.NewDecoder(body).Decode(&loginRequest)
			if err != nil {
				log.Error().Err(err).Msg("Failed to decode login request")
				w.WriteHeader(http.StatusBadRequest)
				return
			}

			token, err := authentificationService.PasswordLogin(loginRequest.Email, loginRequest.Password)

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
	})))

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			log.Info().Msg("Launching GraphQL server")

			go func() {
				if err := http.ListenAndServe("0.0.0.0:9877", nil); err != nil {
					panic(err)
				}
			}()

			log.Info().Msg("☁️  launched GraphQL on port 9877")

			return nil
		},
		OnStop: func(context.Context) error {
			log.Info().Msg("GraphQL server stopped")
			return nil
		},
	})

	return &BrumeHTTPServer{}
}
