package main

import (
	"context"
	"errors"
	"fmt"
	"log"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

const schema = `
CREATE TABLE IF NOT EXISTS messages (
	id         bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name       text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 64),
	text       text NOT NULL CHECK (char_length(text) BETWEEN 1 AND 500),
	created_at timestamptz NOT NULL DEFAULT now()
)`

const (
	// How long one attempt to reach Postgres may take.
	attemptTimeout = 5 * time.Second
	// The longest wait between two attempts.
	maxRetryDelay = 10 * time.Second
)

// postgres is the store in Postgres.
type postgres struct {
	pool *pgxpool.Pool
	// host:port/database, without the user or password, for the logs.
	where string
}

// openPostgres prepares a pool for url, without connecting yet. Its errors never quote url:
// it holds the password.
func openPostgres(ctx context.Context, url string) (*postgres, error) {
	if url == "" {
		return nil, errors.New("DATABASE_URL is not set: expected postgres://user:password@host:5432/db")
	}
	config, err := pgxpool.ParseConfig(url)
	if err != nil {
		return nil, errors.New("DATABASE_URL is not a valid postgres URL: expected postgres://user:password@host:5432/db")
	}
	config.MaxConns = 4
	pool, err := pgxpool.NewWithConfig(ctx, config)
	if err != nil {
		return nil, fmt.Errorf("preparing the postgres pool: %w", err)
	}
	where := fmt.Sprintf("%s:%d/%s", config.ConnConfig.Host, config.ConnConfig.Port, config.ConnConfig.Database)
	return &postgres{pool: pool, where: where}, nil
}

func (p *postgres) close() { p.pool.Close() }

// waitAndMigrate creates the table, retrying with a growing delay until Postgres answers or
// ctx ends.
func (p *postgres) waitAndMigrate(ctx context.Context) error {
	delay := time.Second
	for attempt := 1; ; attempt++ {
		attemptCtx, cancel := context.WithTimeout(ctx, attemptTimeout)
		_, err := p.pool.Exec(attemptCtx, schema)
		cancel()
		if err == nil {
			log.Printf("connected to postgres at %s, table messages ready", p.where)
			return nil
		}
		if ctx.Err() != nil {
			return ctx.Err()
		}
		log.Printf("waiting for postgres at %s (attempt %d, next in %s): %v", p.where, attempt, delay, err)
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(delay):
		}
		delay = min(delay*2, maxRetryDelay)
	}
}

func (p *postgres) list(ctx context.Context, limit int) ([]message, error) {
	rows, err := p.pool.Query(ctx,
		`SELECT id, name, text, created_at FROM messages ORDER BY id DESC LIMIT $1`, limit)
	if err != nil {
		return nil, err
	}
	return pgx.CollectRows(rows, func(row pgx.CollectableRow) (message, error) {
		var m message
		err := row.Scan(&m.ID, &m.Name, &m.Text, &m.CreatedAt)
		return m, err
	})
}

func (p *postgres) add(ctx context.Context, name, text string) (message, error) {
	m := message{Name: name, Text: text}
	err := p.pool.QueryRow(ctx,
		`INSERT INTO messages (name, text) VALUES ($1, $2) RETURNING id, created_at`, name, text,
	).Scan(&m.ID, &m.CreatedAt)
	return m, err
}
