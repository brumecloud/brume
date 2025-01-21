package db_fx

import (
	"github.com/brumecloud/agent/internal/db"
	"go.uber.org/fx"
)

var DBModule = fx.Module("db", fx.Provide(db.InitDB))
