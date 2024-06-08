package fx_grpc

import (
	"brume.dev/internal/router/grpc"
	"go.uber.org/fx"
)

var GRPCModule = fx.Options(
	fx.Provide(grpc_router.NewGRPCRouter),
	fx.Invoke(func(s *grpc_router.GRPCRouter) {}),
)
