package collector

import (
	"github.com/rs/zerolog/log"
)

type CollectorService struct{}

var logger = log.With().Str("module", "collector").Logger()

var PATH_TO_BINARY = "./"

func NewCollectorService() *CollectorService {
	return &CollectorService{}
}

func (c *CollectorService) Start() {
}
