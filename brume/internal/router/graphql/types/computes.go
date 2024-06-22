package graphql_types

import (
	"github.com/graphql-go/graphql"
)

var ComputeType = graphql.NewObject(
	graphql.ObjectConfig{
		Name: "Compute",
		Fields: graphql.Fields{
			"id": &graphql.Field{
				Type: graphql.String,
			},
			"name": &graphql.Field{
				Type: graphql.String,
			},
			"builder": &graphql.Field{
				Type: graphql.String,
			},
		},
	},
)
