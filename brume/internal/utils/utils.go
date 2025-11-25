package brume_utils

import (
	"strings"

	"github.com/google/uuid"
)

func GenerateID(prefix string) string {
	v7, err := uuid.NewV7()
	if err != nil {
		panic(err)
	}

	without := strings.ReplaceAll(v7.String(), "-", "")[:12]

	return prefix + "-" + without
}

func ProjectID() string {
	return GenerateID("proj")
}

func ServiceID() string {
	return GenerateID("serv")
}

func DeploymentID() string {
	return GenerateID("dplm")
}

func JobID() string {
	return GenerateID("job")
}

func RunnerID() string {
	return GenerateID("runr")
}

func BuilderID() string {
	return GenerateID("bldr")
}

func CloudAccountID() string {
	return GenerateID("ca")
}

func UserID() string {
	return GenerateID("usr")
}

func StackID() string {
	return GenerateID("stk")
}

func StackTemplateID() string {
	return GenerateID("stck_tpl")
}
