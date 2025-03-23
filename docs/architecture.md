# Brume Architecture

This document provides an overview of Brume's architecture, explaining how the different components work together.

## High-Level Architecture

Brume is built with a distributed architecture consisting of several key components:

```
                     ┌─────────────┐
                     │   Console   │
                     │  (Web UI)   │
                     └──────┬──────┘
                            │
                            ▼
┌────────────┐      ┌──────────────┐      ┌────────────┐
│   Agent    │◄────►│ Orchestrator │◄────►│   Agent    │
│ (Machine 1)│      │ (Core Logic) │      │ (Machine N)│
└────────────┘      └──────┬───────┘      └────────────┘
                           │
                           ▼
┌───────────┐  ┌──────────┐  ┌───────────┐  ┌───────────┐
│PostgreSQL │  │ClickHouse│  │   Redis   │  │ Temporal  │
└───────────┘  └──────────┘  └───────────┘  └───────────┘
```

## Orchestrator

The Orchestrator is the central component of Brume that manages the entire system. It coordinates job placement, monitors agents, and maintains the system's overall health.

### Bids

The Bidding system in Brume is a core scheduling mechanism that matches jobs with appropriate agents:

- When a service needs to be deployed, the Orchestrator creates a job and makes it available for agents to bid on.
- Agents poll the scheduler for available jobs using the `/job` endpoint.
- Agents evaluate jobs based on their capabilities and resources, then place bids via the `/bid/{bidId}` endpoint.
- Currently, the system uses a first-come-first-served approach, where the first agent to bid gets the job.
- Future enhancements will include sophisticated bidding logic that considers:
  - Network topology
  - Machine type and capabilities
  - Resource availability
  - Historical performance

The bid workflow is managed through Temporal, with updates signaled to the workflow when a machine is found for a job.

### Temporal

Temporal serves as the backbone for workflow orchestration in Brume:

- Manages durable, fault-tolerant workflows that survive process or machine failures
- Brume uses dedicated task queues:
  - `MasterTaskQueue`: For orchestrator-level operations
  - `NodeTaskQueue`: For agent-level operations
- Key workflows include:
  - `DeploymentWorkflow`: Manages the entire lifecycle of service deployments
  - `BidWorkflow`: Handles the job bidding process
  - `MachineHealthCheck`: Monitors agent health
- Activities are atomic units of work executed as part of workflows:
  - `IngestLogs`: Collects and processes logs
  - `StartService`, `StopService`: Control service lifecycle
  - `GetLogs`, `GetStatus`: Retrieve operational data
- Signals like `StopDeploymentSignal` allow for asynchronous communication with running workflows

Temporal provides transactional guarantees and retry mechanisms that enable Brume to maintain system integrity even during failures.

## Agent

Agents are distributed components that run on machines (either customer-owned or in Brume Cloud) and are responsible for executing jobs.

### Runner

The Runner subsystem handles the execution of jobs on agent machines:

- Implements different runtime environments via the `Runner` interface
- Currently supports Docker as the primary container runtime
- Key responsibilities:
  - `StartJob`: Launches a job in the appropriate runner
  - `StopJob`: Gracefully terminates running jobs
  - `GetJobStatus`: Monitors the health and state of jobs
  - `GetJobLogs`: Collects and forwards logs to the Orchestrator
  - `GetRunnerHealth`: Reports the health of the runtime environment
- Agents evaluate their capability to run jobs and place bids accordingly
- Once a bid is accepted, the agent starts and manages the job's lifecycle
- Runners are designed to be pluggable, allowing for future support of different execution environments

### Monitoring

The monitoring subsystem ensures operational visibility:

- Agents send regular health updates to the Orchestrator via dedicated endpoints:
  - `/agent/status`: Reports overall agent health
  - `/jobs/status`: Reports status of all running jobs
- A fast ticker mechanism performs frequent checks:
  - Collects container health metrics
  - Gathers logs from running jobs
  - Updates the Orchestrator on any state changes
- Monitoring data flows into the system's observability stack (ClickHouse)
- This data is used for:
  - Real-time status dashboards in the Console
  - Historical performance analysis
  - Automatic remediation (restarting unhealthy services)
  - Billing and resource usage tracking

The monitoring system operates with minimal overhead, ensuring that agents remain responsive while providing comprehensive visibility.
