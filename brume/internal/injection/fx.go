package injection

import (
	fx_org "brume.dev/account/org/fx"
	fx_user "brume.dev/account/user/fx"
	fx_common "brume.dev/internal/common/fx"
	"brume.dev/internal/db"
	brumelog "brume.dev/internal/log"
	fx_graphql "brume.dev/internal/router/graphql/fx"
	fx_grpc "brume.dev/internal/router/grpc/fx"
	fx_project "brume.dev/project/fx"
	fx_runner "brume.dev/runner/fx"
	fx_service "brume.dev/service/fx"
	"github.com/ipfans/fxlogger"
	"github.com/rs/zerolog/log"
	"go.uber.org/fx"
)

type GlobalInjector struct {
	Injector *fx.App
}

func NewMasterInjector() *GlobalInjector {
	log.Info().Msg("Initializing master injector")

	app := fx.New(
		fx.WithLogger(fxlogger.WithZerolog(brumelog.GetLogger())),
		fx.Provide(db.InitDB),
		fx.Invoke(db.InitDB),

		fx_common.CommonModule,
		fx_org.OrgModule,
		fx_user.UserModule,
		fx_project.ProjectModule,
		fx_service.ServiceModule,
		fx_runner.RunnerModule,

		fx_graphql.GraphQLModule,
		fx_grpc.GRPCModule,
	)

	return &GlobalInjector{
		Injector: app,
	}
}

func (g *GlobalInjector) Run() {
	log.Info().Msg("Running the application")

	g.Injector.Run()
}
