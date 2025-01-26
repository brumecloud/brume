package temporal_constants

const (
	MasterTaskQueue = "master"
	NodeTaskQueue   = "node"
)

// Master worker workflows
const (
	DeploymentWorkflow = "DeploymentWorkflow"
	BidWorkflow        = "BidWorkflow"
	MachineHealthCheck = "MachineHealthCheck"
)

// Master worker activities
const IngestLogs = "IngestLogs"

// Node worker activities
const (
	StartService = "StartService"
	StopService  = "StopService"
	GetLogs      = "GetLogs"
	GetStatus    = "GetStatus"
)

// Signals
const StopDeploymentSignal = "StopDeploymentSignal"

// Jobs
const (
	UnhealthyJobSignal = "UnhealthyJobSignal"
	StopJobSignal      = "StopJobSignal"
)
