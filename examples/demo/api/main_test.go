package main

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"
)

// memoryStore keeps the messages in memory, newest last.
type memoryStore struct {
	mu       sync.Mutex
	messages []message
}

func (m *memoryStore) list(_ context.Context, limit int) ([]message, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	var out []message
	for i := len(m.messages) - 1; i >= 0 && len(out) < limit; i-- {
		out = append(out, m.messages[i])
	}
	return out, nil
}

func (m *memoryStore) add(_ context.Context, name, text string) (message, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	msg := message{ID: int64(len(m.messages) + 1), Name: name, Text: text, CreatedAt: time.Now()}
	m.messages = append(m.messages, msg)
	return msg, nil
}

func newTestServer(ready bool) *server {
	s := &server{store: &memoryStore{}}
	s.ready.Store(ready)
	return s
}

func do(t *testing.T, h http.Handler, method, path, body string) *httptest.ResponseRecorder {
	t.Helper()
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

func TestPostThenList(t *testing.T) {
	h := newTestServer(true).routes()
	for _, name := range []string{"ada", "grace"} {
		rec := do(t, h, "POST", "/api/messages", `{"name":"  `+name+` ","text":"hello\r\nworld"}`)
		if rec.Code != http.StatusCreated {
			t.Fatalf("POST: got %d %s", rec.Code, rec.Body)
		}
	}
	rec := do(t, h, "GET", "/api/messages", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("GET: got %d", rec.Code)
	}
	var got []message
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatal(err)
	}
	if len(got) != 2 || got[0].Name != "grace" || got[1].Name != "ada" || got[0].Text != "hello\nworld" {
		t.Fatalf("GET: got %+v", got)
	}
}

func TestListLimit(t *testing.T) {
	s := newTestServer(true)
	for range 60 {
		s.store.add(context.Background(), "a", "b")
	}
	var got []message
	json.Unmarshal(do(t, s.routes(), "GET", "/api/messages", "").Body.Bytes(), &got)
	if len(got) != listLimit || got[0].ID != 60 {
		t.Fatalf("got %d messages, first %d", len(got), got[0].ID)
	}
}

func TestEmptyListIsAnArray(t *testing.T) {
	rec := do(t, newTestServer(true).routes(), "GET", "/api/messages", "")
	if body := strings.TrimSpace(rec.Body.String()); body != "[]" {
		t.Fatalf("got %q", body)
	}
}

func TestPostRejects(t *testing.T) {
	cases := map[string]string{
		"not json":       `name=a&text=b`,
		"unknown field":  `{"name":"a","text":"b","admin":true}`,
		"two objects":    `{"name":"a","text":"b"}{"name":"a","text":"b"}`,
		"no name":        `{"text":"b"}`,
		"blank name":     `{"name":"   ","text":"b"}`,
		"no text":        `{"name":"a"}`,
		"long name":      `{"name":"` + strings.Repeat("x", 65) + `","text":"b"}`,
		"long text":      `{"name":"a","text":"` + strings.Repeat("é", 501) + `"}`,
		"newline name":   `{"name":"a\nb","text":"b"}`,
		"nul text":       `{"name":"a","text":"a\u0000b"}`,
		"too large body": `{"name":"a","text":"` + strings.Repeat(" ", 20000) + `b"}`,
		"wrong type":     `{"name":1,"text":"b"}`,
	}
	h := newTestServer(true).routes()
	for name, body := range cases {
		rec := do(t, h, "POST", "/api/messages", body)
		if rec.Code != http.StatusBadRequest {
			t.Errorf("%s: got %d %s", name, rec.Code, rec.Body)
		}
	}
}

func TestPostAcceptsTheLimits(t *testing.T) {
	body := `{"name":"` + strings.Repeat("é", 64) + `","text":"` + strings.Repeat("é", 500) + `"}`
	rec := do(t, newTestServer(true).routes(), "POST", "/api/messages", body)
	if rec.Code != http.StatusCreated {
		t.Fatalf("got %d %s", rec.Code, rec.Body)
	}
}

func TestNotReady(t *testing.T) {
	h := newTestServer(false).routes()
	for _, c := range []struct{ method, path, body string }{
		{"GET", "/healthz", ""},
		{"GET", "/api/messages", ""},
		{"POST", "/api/messages", `{"name":"a","text":"b"}`},
	} {
		if rec := do(t, h, c.method, c.path, c.body); rec.Code != http.StatusServiceUnavailable {
			t.Errorf("%s %s: got %d", c.method, c.path, rec.Code)
		}
	}
	if rec := do(t, newTestServer(true).routes(), "GET", "/healthz", ""); rec.Code != http.StatusOK {
		t.Errorf("ready /healthz: got %d", rec.Code)
	}
}

func TestOpenPostgresHidesTheURL(t *testing.T) {
	_, err := openPostgres(context.Background(), "postgres://u:s3cret@db:5432/x?sslmode=bogus", "")
	if err == nil || strings.Contains(err.Error(), "s3cret") {
		t.Fatalf("got %v", err)
	}
}
