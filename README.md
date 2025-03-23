# Brume

> ⚠️ **IMPORTANT**: Brume is currently in early development and is not yet ready for production use. Use at your own risk.

Brume is a modern cloud orchestration platform that allows you to create and manage your own personal cloud infrastructure. It brings cloud computing closer to you, giving you more control over where and how your applications run.

Brume is not a regular orchestrator, compare to all the other orchestrator it is not master / slave. Each machine polls the orchestrator for jobs instead of being dictated by the orchestrator.

## Overview

Brume helps you create, deploy, and manage applications across your own infrastructure. Whether you're running applications on your own hardware, virtual machines, or a mix of cloud providers, Brume provides a unified interface to manage it all.

Key features:

- **Distributed architecture**: The Brume agent can run anywhere
- **Application management**: Deploy and monitor your applications with ease
- **Infrastructure abstraction**: Manage different types of infrastructure through a consistent interface
- **Observability**: Built-in monitoring and logging
- **Developer-friendly**: Modern web console for intuitive management

## Components

Brume consists of several key components:

- **Orchestrator**: The central control plane that manages projects, deployments, and coordinates agents
- **Agent**: Runs on your machines to execute and monitor applications
- **Console**: Web-based user interface for managing your Brume environment
- **CLI**: Command-line interface for interacting with Brume

## Self Hosting

The easiest way to get started with Brume is using Docker Compose:

```bash
docker-compose up -d
```

This will start all necessary services, including:

- Brume Orchestrator
- Brume Agent
- Web Console
- Supporting services (PostgreSQL, Temporal, Redis, ClickHouse)

Once everything is running, you can access:

- Web Console: http://localhost:3000
- Temporal UI: http://localhost:8080

## Development

See the [development documentation](./docs/development.md) for information on setting up a development environment.

## Documentation

For detailed documentation, please see the [docs](./docs) directory.
