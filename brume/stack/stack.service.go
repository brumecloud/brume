package stack_service

import (
	"context"

	cloud_account_service "brume.dev/cloud/account"
	"brume.dev/internal/db"
	temporal_client "brume.dev/internal/durable/client"
	"brume.dev/internal/log"
	brume_utils "brume.dev/internal/utils"
	stack_interfaces "brume.dev/stack/durable"
	workflow_stack "brume.dev/stack/durable/workflow"
	stack_model "brume.dev/stack/model"
	"go.temporal.io/sdk/client"
)

var logger = log.GetLogger("stack.service")

type StackService struct {
	db                  *db.DB
	cloudAccountService *cloud_account_service.CloudAccountService
	temporalClient      *temporal_client.TemporalClient
}

func NewStackService(db *db.DB, cloudAccountService *cloud_account_service.CloudAccountService, temporalClient *temporal_client.TemporalClient) *StackService {
	return &StackService{db: db, cloudAccountService: cloudAccountService, temporalClient: temporalClient}
}

// TODO: authZ
func (s *StackService) GetStackTemplateByID(ctx context.Context, id string) (*stack_model.StackTemplate, error) {
	var template stack_model.StackTemplate
	err := s.db.Gorm.Where("id = ?", id).First(&template).Error
	if err != nil {
		logger.Warn().Err(err).Str("id", id).Msg("Failed to get stack template by id")
		return nil, err
	}
	return &template, nil
}

func (s *StackService) GetAllStacks(ctx context.Context) ([]*stack_model.Stack, error) {
	var stacks []*stack_model.Stack
	err := s.db.Gorm.Find(&stacks).Error
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to get all stacks")
		return nil, err
	}
	return stacks, nil
}

func (s *StackService) GetAllStackTemplates(ctx context.Context) ([]*stack_model.StackTemplate, error) {
	var templates []*stack_model.StackTemplate
	err := s.db.Gorm.Find(&templates).Error
	if err != nil {
		logger.Warn().Err(err).Msg("Failed to get all stack templates")
		return nil, err
	}
	return templates, nil
}

func (s *StackService) GetStackByID(ctx context.Context, id string) (*stack_model.Stack, error) {
	var stack stack_model.Stack
	err := s.db.Gorm.Where("id = ?", id).First(&stack).Error
	if err != nil {
		logger.Warn().Err(err).Str("id", id).Msg("Failed to get stack by id")
		return nil, err
	}
	return &stack, nil
}

func (s *StackService) UpdateStack(ctx context.Context, stack *stack_model.Stack) (*stack_model.Stack, error) {
	err := s.db.Gorm.Save(stack).Error
	if err != nil {
		logger.Error().Err(err).Str("stack_id", stack.ID).Msg("Failed to update the stack in the database")
		return nil, err
	}
	return stack, nil
}

type StartStackDeploymentInput struct {
	Name           string
	TemplateID     string
	CloudAccountID string
}

// start the deployment of the stack by starting the stack workflow
func (s *StackService) StartStackDeployment(ctx context.Context, input StartStackDeploymentInput) (*stack_model.Stack, error) {
	stackTemplate, err := s.GetStackTemplateByID(ctx, input.TemplateID)
	if err != nil {
		logger.Warn().Err(err).Str("id", input.TemplateID).Msg("Failed to get stack template by id")
		return nil, err
	}

	account, err := s.cloudAccountService.GetCloudAccountByID(ctx, input.CloudAccountID)
	if err != nil {
		logger.Warn().Err(err).Str("id", input.CloudAccountID).Msg("Failed to get cloud account by id")
		return nil, err
	}

	// create the empty stack, start the stack deployment workflow and immediately return the stack id
	stack := &stack_model.Stack{
		ID:             brume_utils.StackID(),
		Name:           input.Name,
		TemplateID:     stackTemplate.ID,
		CloudAccountID: input.CloudAccountID,
		Status:         stack_model.StackStatusPending,
	}

	err = s.db.Gorm.Create(stack).Error
	if err != nil {
		logger.Error().Err(err).Str("stack_id", stack.ID).Msg("Failed to save the stack in the database")
		return nil, err
	}

	// // start the stack deployment workflow
	opts := client.StartWorkflowOptions{
		ID:        "stack-deployment-workflow-" + stack.ID,
		TaskQueue: temporal_client.DefaultTaskQueue,
	}
	_, err = s.temporalClient.Client.ExecuteWorkflow(ctx, opts, workflow_stack.CreateStackWorkflow, stack_interfaces.CreateStackWorkflowInput{
		Name:         input.Name,
		Stack:        stack,
		CloudAccount: account,
	})

	if err != nil {
		logger.Error().Err(err).Str("stack_id", stack.ID).Msg("Failed to start the stack deployment workflow")
		return nil, err
	}

	return stack, nil
}
