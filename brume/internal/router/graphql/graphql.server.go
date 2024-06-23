package graphql_router

import (
	"context"
	"encoding/json"
	"io"
	"net"
	"net/http"
	"os"
	"time"

	"brume.dev/internal/common"
	resolver "brume.dev/internal/router/graphql/resolver"
	graphql "github.com/graph-gophers/graphql-go"
	"github.com/graph-gophers/graphql-go/relay"
	"github.com/rs/cors"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

// for fx
type GraphQLServer struct{}

type LoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

func readGraphQLSchema() string {
	// open file scheam.graphql
	schemaFile, err := os.Open("./internal/router/graphql/schema.graphql")

	if err != nil {
		log.Panic().Err(err).Msg("Failed to open schema file")
		panic(err)
	}

	defer schemaFile.Close()
	schemaFileContent, err := io.ReadAll(schemaFile)

	if err != nil {
		log.Panic().Err(err).Msg("Failed to read schema file")
		panic(err)
	}

	return string(schemaFileContent)
}

func AuthMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if len(r.Cookies()) == 0 || r.Cookies()[0].Name != "access_token" {
			w.WriteHeader(http.StatusUnauthorized)
			return
		}

		token := r.Cookies()[0].Value
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

func NewGraphQLServer(lc fx.Lifecycle, authentificationService *common.AuthentificationService, rootResolver *resolver.RootResolver) *GraphQLServer {
	log.Info().Msg("Creating the GraphQL server")

	schemaFileContent := readGraphQLSchema()

	schema := graphql.MustParseSchema(schemaFileContent, rootResolver)

	cors := cors.New(cors.Options{
		AllowedOrigins:   []string{"http://localhost:5173"},
		AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders:   []string{"Accept", "Authorization", "Content-Type"},
		AllowCredentials: true,
		MaxAge:           300,
	})

	// graphql server
	http.Handle("/graphql", cors.Handler(AuthMiddleware((&relay.Handler{
		Schema: schema,
	}))))

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

	// graphiql server
	http.Handle("/", http.FileServer(http.Dir("./static/graphiql")))

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			log.Info().Msg("Launching GraphQL server")

			var lis net.Listener
			lis, err := net.Listen("tcp", "localhost:9877")

			if err != nil {
				panic(err)
			}

			go func() {
				if err := http.Serve(lis, nil); err != nil {
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

	return &GraphQLServer{}
}
