package collector

import (
	"bufio"
	"context"
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

func NewOtelBin() *OtelBin {
	return &OtelBin{
		// use the logger from the collector service
		logger: &logger,
		doneCh: make(chan struct{}),
		args:   []string{"--config", "/brume/agent/collector/otel.yaml"},
	}
}

func (b *OtelBin) Start() error {
	logger.Info().Msg("Starting otel collector")

	b.cmd = exec.CommandContext(context.Background(), PATH_TO_BINARY, b.args...)

	stdoutPipe, err := b.cmd.StdoutPipe()
	if err != nil {
		return err
	}

	stderrPipe, err := b.cmd.StderrPipe()
	if err != nil {
		return err
	}

	go func() {
		scanner := bufio.NewScanner(stdoutPipe)
		for scanner.Scan() {
			b.logger.Info().Msg(scanner.Text())
		}
	}()

	go func() {
		scanner := bufio.NewScanner(stderrPipe)
		for scanner.Scan() {
			b.logger.Warn().Msg(scanner.Text())
		}
	}()

	b.waitCh = make(chan struct{}, 1)
	b.doneCh = make(chan struct{})

	if err := b.cmd.Start(); err != nil {
		logger.Error().Err(err).Msg("Failed to start otel collector")
	}

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

	if err := b.cmd.Process.Kill(); err != nil {
		logger.Error().Err(err).Msg("Failed to kill otel collector")
		return err
	}

	return nil
}
