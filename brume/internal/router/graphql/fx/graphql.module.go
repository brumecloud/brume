package fx_graphql

import (
	"brume.dev/internal/router/graphql"
	resolver "brume.dev/internal/router/graphql/resolver"
	"go.uber.org/fx"
)

var GraphQLModule = fx.Options(
	fx.Provide(graphql_router.NewGraphQLServer, resolver.NewRootResolver),
	fx.Invoke(func(s *graphql_router.GraphQLServer) {}),
)
