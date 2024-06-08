package graphql_schema

import (
	"github.com/graphql-go/graphql"
)

func NewSchema() (graphql.Schema, error) {
	fields := graphql.Fields{
		"hello": &graphql.Field{
			Type: graphql.String,
			Resolve: func(p graphql.ResolveParams) (interface{}, error) {
				return "Hello World!", nil
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
