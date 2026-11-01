// Command api is the guestbook API of the Brume demo. It keeps the messages in Postgres.
//
// It listens on :8080 at once, then waits for Postgres in the background: Postgres may
// start after it. Until the table is ready, /healthz and /api/ answer 503.
package main

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"sync/atomic"
	"syscall"
	"time"
	"unicode"
	"unicode/utf8"
)

const (
	listenAddr = ":8080"
	// The longest name and text, in characters, as Postgres' char_length counts them.
	maxNameChars = 64
	maxTextChars = 500
	// Room for 500 characters escaped as \uXXXX, and then some.
	maxBodyBytes = 16 << 10
	// How many messages GET /api/messages returns, newest first.
	listLimit = 50
)

type message struct {
	ID        int64     `json:"id"`
	Name      string    `json:"name"`
	Text      string    `json:"text"`
	CreatedAt time.Time `json:"created_at"`
}

// store keeps the messages.
type store interface {
	list(ctx context.Context, limit int) ([]message, error)
	add(ctx context.Context, name, text string) (message, error)
}

type server struct {
	store store
	// Set once the table exists: before, the API answers 503.
	ready atomic.Bool
}

func (s *server) routes() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /healthz", s.healthz)
	mux.HandleFunc("GET /api/messages", s.listMessages)
	mux.HandleFunc("POST /api/messages", s.addMessage)
	return logRequests(mux)
}

func (s *server) healthz(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "text/plain; charset=utf-8")
	if !s.ready.Load() {
		w.WriteHeader(http.StatusServiceUnavailable)
		io.WriteString(w, "waiting for postgres\n")
		return
	}
	io.WriteString(w, "ok\n")
}

func (s *server) listMessages(w http.ResponseWriter, r *http.Request) {
	if !s.ready.Load() {
		writeError(w, http.StatusServiceUnavailable, "the database is not ready yet")
		return
	}
	messages, err := s.store.list(r.Context(), listLimit)
	if err != nil {
		log.Printf("listing messages: %v", err)
		writeError(w, http.StatusInternalServerError, "database error")
		return
	}
	if messages == nil {
		messages = []message{}
	}
	writeJSON(w, http.StatusOK, messages)
}

func (s *server) addMessage(w http.ResponseWriter, r *http.Request) {
	if !s.ready.Load() {
		writeError(w, http.StatusServiceUnavailable, "the database is not ready yet")
		return
	}
	var in struct {
		Name string `json:"name"`
		Text string `json:"text"`
	}
	dec := json.NewDecoder(http.MaxBytesReader(w, r.Body, maxBodyBytes))
	dec.DisallowUnknownFields()
	if err := dec.Decode(&in); err != nil {
		writeError(w, http.StatusBadRequest, `the body must be a JSON object {"name", "text"} of at most 16 KiB`)
		return
	}
	if err := dec.Decode(&struct{}{}); !errors.Is(err, io.EOF) {
		writeError(w, http.StatusBadRequest, "the body must hold a single JSON object")
		return
	}
	name, text, err := validate(in.Name, in.Text)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}
	created, err := s.store.add(r.Context(), name, text)
	if err != nil {
		log.Printf("adding a message: %v", err)
		writeError(w, http.StatusInternalServerError, "database error")
		return
	}
	writeJSON(w, http.StatusCreated, created)
}

// validate trims name and text, and checks their length and characters. The text may span
// lines; the name may not.
func validate(name, text string) (string, string, error) {
	name = strings.TrimSpace(name)
	text = strings.TrimSpace(strings.ReplaceAll(text, "\r\n", "\n"))
	switch n := utf8.RuneCountInString(name); {
	case n == 0:
		return "", "", errors.New("name is required")
	case n > maxNameChars:
		return "", "", errors.New("name is longer than 64 characters")
	}
	switch n := utf8.RuneCountInString(text); {
	case n == 0:
		return "", "", errors.New("text is required")
	case n > maxTextChars:
		return "", "", errors.New("text is longer than 500 characters")
	}
	if strings.IndexFunc(name, unicode.IsControl) >= 0 {
		return "", "", errors.New("name holds a control character")
	}
	if strings.IndexFunc(text, func(r rune) bool { return unicode.IsControl(r) && r != '\n' && r != '\t' }) >= 0 {
		return "", "", errors.New("text holds a control character")
	}
	return name, text, nil
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.Header().Set("X-Content-Type-Options", "nosniff")
	w.Header().Set("Cache-Control", "no-store")
	w.WriteHeader(status)
	if err := json.NewEncoder(w).Encode(v); err != nil {
		log.Printf("writing the response: %v", err)
	}
}

func writeError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, map[string]string{"error": msg})
}

// statusRecorder remembers the status and size of a response, for the log.
type statusRecorder struct {
	http.ResponseWriter
	status int
	bytes  int
}

func (r *statusRecorder) WriteHeader(status int) {
	if r.status == 0 {
		r.status = status
	}
	r.ResponseWriter.WriteHeader(status)
}

func (r *statusRecorder) Write(b []byte) (int, error) {
	if r.status == 0 {
		r.status = http.StatusOK
	}
	n, err := r.ResponseWriter.Write(b)
	r.bytes += n
	return n, err
}

// logRequests logs one line per request. The path is logged escaped, so that a request
// cannot write a line of its own.
func logRequests(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		rec := &statusRecorder{ResponseWriter: w}
		next.ServeHTTP(rec, r)
		if rec.status == 0 {
			rec.status = http.StatusOK
		}
		log.Printf("%s %s %d %dB %s", r.Method, r.URL.EscapedPath(), rec.status, rec.bytes,
			time.Since(start).Round(time.Microsecond))
	})
}

func main() {
	log.SetFlags(log.Ldate | log.Ltime | log.LUTC)
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	db, err := openPostgres(ctx, os.Getenv("DATABASE_URL"), os.Getenv("POSTGRES_PASSWORD"))
	if err != nil {
		log.Fatal(err)
	}
	defer db.close()

	s := &server{store: db}
	httpServer := &http.Server{
		Handler:           s.routes(),
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       10 * time.Second,
		WriteTimeout:      10 * time.Second,
		IdleTimeout:       60 * time.Second,
	}
	listener, err := net.Listen("tcp", listenAddr)
	if err != nil {
		log.Fatalf("listening on %s: %v", listenAddr, err)
	}
	log.Printf("brume demo api: listening on %s", listenAddr)
	go func() {
		if err := db.waitAndMigrate(ctx); err == nil {
			s.ready.Store(true)
		}
	}()
	go func() {
		if err := httpServer.Serve(listener); !errors.Is(err, http.ErrServerClosed) {
			log.Fatalf("serving: %v", err)
		}
	}()

	<-ctx.Done()
	log.Print("shutting down")
	shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	if err := httpServer.Shutdown(shutdownCtx); err != nil {
		log.Printf("shutting down: %v", err)
	}
}
