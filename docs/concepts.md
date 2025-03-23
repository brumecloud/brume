# Brume Concepts

This document explains the key concepts and terminology used in Brume.

## Projects

A **Project** is the top-level organizational unit in Brume. It represents a complete application or service that you want to deploy and manage. Projects contain all the information needed to run your software, including:

- Configuration
- Environment variables
- Resource requirements
- Deployment targets
- Scaling policies

Projects are isolated from each other, each with their own configuration and resources.

## Deployments

A **Deployment** represents a specific version of your project running in a specific environment. Deployments are created when you deploy your project to an environment. Key characteristics of deployments include:

- They are immutable representations of your project at a point in time
- Each deployment has a unique identifier
- Deployments track the state of your running application
- They can be rolled back to previous versions

Brume supports different deployment strategies:

- **Rolling updates**: Gradually replace instances of the old version
- **Blue/Green**: Deploy new version alongside the old one, then switch
- **Canary**: Gradually route traffic to the new version

## Machines

A **Machine** in Brume represents a physical or virtual server where your applications can run. Machines:

- Can be physical servers, VMs, or cloud instances
- Run the Brume Agent
- Execute your applications in runners
- Report status, metrics, and logs

Machines can be grouped into clusters for better organization and targeting.

## Services

A **Service** is a specific component of your application that performs a distinct function. Services:

- Have their own configuration
- Can scale independently
- May depend on other services
- Can expose ports for communication

Examples of services include web servers, databases, message queues, or background workers.

## Runners

A **Runner** is the execution environment for your services. Brume supports different types of runners:

- **Container Runner**: Executes services in Docker containers
- **Process Runner**: Runs services as native processes
- **VM Runner**: Manages virtual machines (future)
- **Kubernetes Runner**: Deploys to Kubernetes clusters (future)

Each runner type has its own configuration options and capabilities.

## Configuration

Brume uses a declarative configuration approach. Project configuration can be defined in:

- YAML or TOML files
- The Brume Console UI
- Via the CLI

Configuration includes:

- Resource requirements (CPU, memory)
- Network settings
- Environment variables
- Volumes and storage
- Health checks
- Scaling policies

## Environments

An **Environment** represents a specific stage in your development lifecycle, such as:

- Development
- Staging
- Production

Environments can have different configurations, resources, and access controls.

## Networks

Brume manages networking between your services through:

- **Service Discovery**: Automatically register and discover services
- **Load Balancing**: Distribute traffic across service instances
- **Ingress**: Expose services to external traffic
- **Private Networks**: Secure communication between services

## Observability

Brume provides comprehensive observability for your applications:

- **Metrics**: Collect and visualize performance metrics
- **Logging**: Centralized log collection and analysis
- **Tracing**: Distributed tracing for request flows
- **Alerts**: Notify when issues are detected

## Security

Security features in Brume include:

- **Authentication**: Secure access to the Brume platform
- **Authorization**: Role-based access control
- **Secrets Management**: Securely store and distribute secrets
- **Network Policies**: Control traffic between services

## Workflows

Brume uses Temporal for orchestrating complex, long-running processes such as:

- Deployments
- Scaling operations
- Backup and restore
- Infrastructure provisioning

Workflows are reliable, fault-tolerant, and can handle failures gracefully.
