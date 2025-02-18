package collector

import (
	"context"
	"os"
	"os/exec"
	"sync/atomic"

	"github.com/rs/zerolog"
)

type OtelBin struct {
	logger  *zerolog.Logger
	args    []string
	cmd     *exec.Cmd
	doneCh  chan struct{}
	waitCh  chan struct{}
	running int64
}

const DEFAULT_BINARY_PATH = "./bin/otelcol"

func NewOtelBin() *OtelBin {
	return &OtelBin{
		// use the logger from the collector service
		logger: &logger,
		doneCh: make(chan struct{}),
	}
}

func (b *OtelBin) Start() error {
	logger.Info().Msg("Starting otel collector")

	b.cmd = exec.CommandContext(context.Background(), DEFAULT_BINARY_PATH, b.args...)

	b.cmd.Stdout = os.Stdout
	b.cmd.Stderr = os.Stderr

	b.waitCh = make(chan struct{}, 1)
	b.doneCh = make(chan struct{})

	if err := b.cmd.Start(); err != nil {
		logger.Error().Err(err).Msg("Failed to start otel collector")
	}

	logger.Info().Msg("Otel collector started")
	atomic.StoreInt64(&b.running, 1)

	go b.watch()

	return nil
}

func (b *OtelBin) watch() {
	b.cmd.Wait()
	b.doneCh <- struct{}{}

	atomic.StoreInt64(&b.running, 0)
	close(b.waitCh)
}

func (b *OtelBin) Stop() error {
	if b.cmd == nil || b.cmd.Process == nil {
		return nil
	}

	logger.Info().Msg("Stopping otel collector")

	if err := b.cmd.Process.Kill(); err != nil {
		logger.Error().Err(err).Msg("Failed to kill otel collector")
		return err
	}

	return nil
}
