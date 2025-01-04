package brume_redis

import (
	"context"
	"fmt"

	config "brume.dev/internal/config"
	redis "github.com/redis/go-redis/v9"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

var logger = log.With().Str("module", "redis").Logger()

func NewRedisClient(cfg *config.BrumeConfig) *redis.Client {
	client := redis.NewClient(&redis.Options{
		Addr:     fmt.Sprintf("%s:%d", cfg.RedisHost, cfg.RedisPort),
		Password: cfg.RedisPassword,
		DB:       cfg.RedisDB,
	})

	_, err := client.Ping(context.Background()).Result()
	logger.Info().Msg("Connected to Redis")

	if err != nil {
		logger.Error().Err(err).Msg("Failed to connect to Redis")
		panic(err)
	}

	return client
}

var RedisModule = fx.Module("redis", fx.Provide(NewRedisClient))
