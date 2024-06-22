package graphql_types

import "github.com/graphql-go/graphql"

var ProjectType = graphql.NewObject(
	graphql.ObjectConfig{
		Name: "Project",
		Fields: graphql.Fields{
			"id": &graphql.Field{
				Type: graphql.String,
			},
			"name": &graphql.Field{
				Type: graphql.String,
			},
			"description": &graphql.Field{
				Type: graphql.String,
			},
			"projectVariables": &graphql.Field{
				Type: graphql.String,
			},
		},
	},
)
