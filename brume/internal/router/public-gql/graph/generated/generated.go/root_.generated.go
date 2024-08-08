// Code generated by github.com/99designs/gqlgen, DO NOT EDIT.

package public_graph_generated

import (
	"bytes"
	"context"
	"errors"
	"sync/atomic"

	public_graph_model "brume.dev/internal/router/public-gql/graph/model"
	"github.com/99designs/gqlgen/graphql"
	"github.com/99designs/gqlgen/graphql/introspection"
	gqlparser "github.com/vektah/gqlparser/v2"
	"github.com/vektah/gqlparser/v2/ast"
)

// NewExecutableSchema creates an ExecutableSchema from the ResolverRoot interface.
func NewExecutableSchema(cfg Config) graphql.ExecutableSchema {
	return &executableSchema{
		schema:     cfg.Schema,
		resolvers:  cfg.Resolvers,
		directives: cfg.Directives,
		complexity: cfg.Complexity,
	}
}

type Config struct {
	Schema     *ast.Schema
	Resolvers  ResolverRoot
	Directives DirectiveRoot
	Complexity ComplexityRoot
}

type ResolverRoot interface {
	Log() LogResolver
	Mutation() MutationResolver
	Project() ProjectResolver
	Query() QueryResolver
	Service() ServiceResolver
	Subscription() SubscriptionResolver
	User() UserResolver
}

type DirectiveRoot struct {
}

type ComplexityRoot struct {
	Log struct {
		ID        func(childComplexity int) int
		Level     func(childComplexity int) int
		Message   func(childComplexity int) int
		Timestamp func(childComplexity int) int
	}

	Mutation struct {
		AddServiceToProject func(childComplexity int, projectID string, input public_graph_model.CreateServiceInput) int
		CreateProject       func(childComplexity int, name string, description *string) int
	}

	Project struct {
		Description func(childComplexity int) int
		ID          func(childComplexity int) int
		Name        func(childComplexity int) int
		Services    func(childComplexity int) int
	}

	Query struct {
		GetProjectByID func(childComplexity int, id string) int
		Me             func(childComplexity int) int
		ServiceLogs    func(childComplexity int, serviceID string) int
	}

	Service struct {
		ID   func(childComplexity int) int
		Name func(childComplexity int) int
	}

	Subscription struct {
		ServiceLogs func(childComplexity int, serviceID string) int
	}

	User struct {
		Avatar   func(childComplexity int) int
		ID       func(childComplexity int) int
		Name     func(childComplexity int) int
		Projects func(childComplexity int) int
	}
}

type executableSchema struct {
	schema     *ast.Schema
	resolvers  ResolverRoot
	directives DirectiveRoot
	complexity ComplexityRoot
}

func (e *executableSchema) Schema() *ast.Schema {
	if e.schema != nil {
		return e.schema
	}
	return parsedSchema
}

func (e *executableSchema) Complexity(typeName, field string, childComplexity int, rawArgs map[string]interface{}) (int, bool) {
	ec := executionContext{nil, e, 0, 0, nil}
	_ = ec
	switch typeName + "." + field {

	case "Log.id":
		if e.complexity.Log.ID == nil {
			break
		}

		return e.complexity.Log.ID(childComplexity), true

	case "Log.level":
		if e.complexity.Log.Level == nil {
			break
		}

		return e.complexity.Log.Level(childComplexity), true

	case "Log.message":
		if e.complexity.Log.Message == nil {
			break
		}

		return e.complexity.Log.Message(childComplexity), true

	case "Log.timestamp":
		if e.complexity.Log.Timestamp == nil {
			break
		}

		return e.complexity.Log.Timestamp(childComplexity), true

	case "Mutation.addServiceToProject":
		if e.complexity.Mutation.AddServiceToProject == nil {
			break
		}

		args, err := ec.field_Mutation_addServiceToProject_args(context.TODO(), rawArgs)
		if err != nil {
			return 0, false
		}

		return e.complexity.Mutation.AddServiceToProject(childComplexity, args["projectId"].(string), args["input"].(public_graph_model.CreateServiceInput)), true

	case "Mutation.createProject":
		if e.complexity.Mutation.CreateProject == nil {
			break
		}

		args, err := ec.field_Mutation_createProject_args(context.TODO(), rawArgs)
		if err != nil {
			return 0, false
		}

		return e.complexity.Mutation.CreateProject(childComplexity, args["name"].(string), args["description"].(*string)), true

	case "Project.description":
		if e.complexity.Project.Description == nil {
			break
		}

		return e.complexity.Project.Description(childComplexity), true

	case "Project.id":
		if e.complexity.Project.ID == nil {
			break
		}

		return e.complexity.Project.ID(childComplexity), true

	case "Project.name":
		if e.complexity.Project.Name == nil {
			break
		}

		return e.complexity.Project.Name(childComplexity), true

	case "Project.services":
		if e.complexity.Project.Services == nil {
			break
		}

		return e.complexity.Project.Services(childComplexity), true

	case "Query.getProjectById":
		if e.complexity.Query.GetProjectByID == nil {
			break
		}

		args, err := ec.field_Query_getProjectById_args(context.TODO(), rawArgs)
		if err != nil {
			return 0, false
		}

		return e.complexity.Query.GetProjectByID(childComplexity, args["id"].(string)), true

	case "Query.me":
		if e.complexity.Query.Me == nil {
			break
		}

		return e.complexity.Query.Me(childComplexity), true

	case "Query.serviceLogs":
		if e.complexity.Query.ServiceLogs == nil {
			break
		}

		args, err := ec.field_Query_serviceLogs_args(context.TODO(), rawArgs)
		if err != nil {
			return 0, false
		}

		return e.complexity.Query.ServiceLogs(childComplexity, args["serviceId"].(string)), true

	case "Service.id":
		if e.complexity.Service.ID == nil {
			break
		}

		return e.complexity.Service.ID(childComplexity), true

	case "Service.name":
		if e.complexity.Service.Name == nil {
			break
		}

		return e.complexity.Service.Name(childComplexity), true

	case "Subscription.serviceLogs":
		if e.complexity.Subscription.ServiceLogs == nil {
			break
		}

		args, err := ec.field_Subscription_serviceLogs_args(context.TODO(), rawArgs)
		if err != nil {
			return 0, false
		}

		return e.complexity.Subscription.ServiceLogs(childComplexity, args["serviceId"].(string)), true

	case "User.avatar":
		if e.complexity.User.Avatar == nil {
			break
		}

		return e.complexity.User.Avatar(childComplexity), true

	case "User.id":
		if e.complexity.User.ID == nil {
			break
		}

		return e.complexity.User.ID(childComplexity), true

	case "User.name":
		if e.complexity.User.Name == nil {
			break
		}

		return e.complexity.User.Name(childComplexity), true

	case "User.projects":
		if e.complexity.User.Projects == nil {
			break
		}

		return e.complexity.User.Projects(childComplexity), true

	}
	return 0, false
}

func (e *executableSchema) Exec(ctx context.Context) graphql.ResponseHandler {
	rc := graphql.GetOperationContext(ctx)
	ec := executionContext{rc, e, 0, 0, make(chan graphql.DeferredResult)}
	inputUnmarshalMap := graphql.BuildUnmarshalerMap(
		ec.unmarshalInputCreateServiceInput,
	)
	first := true

	switch rc.Operation.Operation {
	case ast.Query:
		return func(ctx context.Context) *graphql.Response {
			var response graphql.Response
			var data graphql.Marshaler
			if first {
				first = false
				ctx = graphql.WithUnmarshalerMap(ctx, inputUnmarshalMap)
				data = ec._Query(ctx, rc.Operation.SelectionSet)
			} else {
				if atomic.LoadInt32(&ec.pendingDeferred) > 0 {
					result := <-ec.deferredResults
					atomic.AddInt32(&ec.pendingDeferred, -1)
					data = result.Result
					response.Path = result.Path
					response.Label = result.Label
					response.Errors = result.Errors
				} else {
					return nil
				}
			}
			var buf bytes.Buffer
			data.MarshalGQL(&buf)
			response.Data = buf.Bytes()
			if atomic.LoadInt32(&ec.deferred) > 0 {
				hasNext := atomic.LoadInt32(&ec.pendingDeferred) > 0
				response.HasNext = &hasNext
			}

			return &response
		}
	case ast.Mutation:
		return func(ctx context.Context) *graphql.Response {
			if !first {
				return nil
			}
			first = false
			ctx = graphql.WithUnmarshalerMap(ctx, inputUnmarshalMap)
			data := ec._Mutation(ctx, rc.Operation.SelectionSet)
			var buf bytes.Buffer
			data.MarshalGQL(&buf)

			return &graphql.Response{
				Data: buf.Bytes(),
			}
		}
	case ast.Subscription:
		next := ec._Subscription(ctx, rc.Operation.SelectionSet)

		var buf bytes.Buffer
		return func(ctx context.Context) *graphql.Response {
			buf.Reset()
			data := next(ctx)

			if data == nil {
				return nil
			}
			data.MarshalGQL(&buf)

			return &graphql.Response{
				Data: buf.Bytes(),
			}
		}

	default:
		return graphql.OneShot(graphql.ErrorResponse(ctx, "unsupported GraphQL operation"))
	}
}

type executionContext struct {
	*graphql.OperationContext
	*executableSchema
	deferred        int32
	pendingDeferred int32
	deferredResults chan graphql.DeferredResult
}

func (ec *executionContext) processDeferredGroup(dg graphql.DeferredGroup) {
	atomic.AddInt32(&ec.pendingDeferred, 1)
	go func() {
		ctx := graphql.WithFreshResponseContext(dg.Context)
		dg.FieldSet.Dispatch(ctx)
		ds := graphql.DeferredResult{
			Path:   dg.Path,
			Label:  dg.Label,
			Result: dg.FieldSet,
			Errors: graphql.GetErrors(ctx),
		}
		// null fields should bubble up
		if dg.FieldSet.Invalids > 0 {
			ds.Result = graphql.Null
		}
		ec.deferredResults <- ds
	}()
}

func (ec *executionContext) introspectSchema() (*introspection.Schema, error) {
	if ec.DisableIntrospection {
		return nil, errors.New("introspection disabled")
	}
	return introspection.WrapSchema(ec.Schema()), nil
}

func (ec *executionContext) introspectType(name string) (*introspection.Type, error) {
	if ec.DisableIntrospection {
		return nil, errors.New("introspection disabled")
	}
	return introspection.WrapTypeFromDef(ec.Schema(), ec.Schema().Types[name]), nil
}

var sources = []*ast.Source{
	{Name: "../../public.graphql", Input: `# public brume GQL schema
type User {
  id: String!
  name: String!
  avatar: String!
  projects: [Project!]!
}

type Project {
  id: String!
  name: String!
  description: String!
  services: [Service!]!
}

type Service {
  id: String!
  name: String!
}

type Log {
  id: String!
  message: String!
  level: String!
  timestamp: String!
}

input CreateServiceInput {
  name: String!
  image: String!
}

type Query {
  me: User!
  getProjectById(id: String!): Project!
  serviceLogs(serviceId: String!): [Log]!
}

type Mutation {
  createProject(name: String!, description: String): Project!
  addServiceToProject(projectId: String!, input: CreateServiceInput!): Service!
}

type Subscription {
  serviceLogs(serviceId: String!): [Log]!
}
`, BuiltIn: false},
}
var parsedSchema = gqlparser.MustLoadSchema(sources...)
