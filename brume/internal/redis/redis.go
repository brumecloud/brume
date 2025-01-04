package brume_redis

import (
	"context"
	"fmt"

	config "brume.dev/internal/config"
	brume_log "brume.dev/internal/log"
	redis "github.com/redis/go-redis/v9"
	"go.uber.org/fx"
)

var logger = brume_log.GetLogger("redis")

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
