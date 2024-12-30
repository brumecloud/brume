package temporal_constants

const MasterTaskQueue = "master"
const NodeTaskQueue = "node"

// Master worker workflows
const DeploymentWorkflow = "DeploymentWorkflow"
const BidWorkflow = "BidWorkflow"

// Master worker activities
const IngestLogs = "IngestLogs"

// Node worker activities
const StartService = "StartService"
const StopService = "StopService"
const GetLogs = "GetLogs"
const GetStatus = "GetStatus"

// Signals
const StopDeploymentSignal = "StopDeploymentSignal"
