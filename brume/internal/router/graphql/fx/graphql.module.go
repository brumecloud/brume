package fx_graphql

import (
	"brume.dev/internal/router/graphql"
	"go.uber.org/fx"
)

var GraphQLModule = fx.Options(
	fx.Provide(graphql_router.NewGraphQLServer),
	fx.Invoke(func(s *graphql_router.GraphQLServer) {}),
)
