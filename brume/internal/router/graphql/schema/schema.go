package graphql_schema

import (
	"brume.dev/internal/router/graphql/types"
	"github.com/graphql-go/graphql"
)

func NewSchema() (graphql.Schema, error) {
	fields := graphql.Fields{
		"project": &graphql.Field{
			Type: graphql.NewList(graphql_types.ProjectType),
			Resolve: func(p graphql.ResolveParams) (interface{}, error) {
				return make([]interface{}, 0), nil
			},
		},
		"compute": &graphql.Field{
			Type: graphql.NewList(graphql_types.ComputeType),
			Resolve: func(p graphql.ResolveParams) (interface{}, error) {
				return make([]interface{}, 0), nil
			},
		},
	}

	rootQuery := graphql.ObjectConfig{
		Name:   "Query",
		Fields: fields,
	}

	schemaConfig := graphql.SchemaConfig{
		Query: graphql.NewObject(rootQuery),
	}

	return graphql.NewSchema(schemaConfig)
}
