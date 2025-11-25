package stack_interfaces

import (
	cloud_account_model "brume.dev/cloud/account/model"
	stack_model "brume.dev/stack/model"
)

type CreateStackWorkflowInput struct {
	Name         string
	Stack        *stack_model.Stack
	CloudAccount *cloud_account_model.CloudAccount
}

var UpdateStackStatusActivityName = "UpdateStackStatus"

type UpdateStackStatusInput struct {
	StackId string
	Status  stack_model.StackStatus
}
