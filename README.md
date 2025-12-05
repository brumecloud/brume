# Brume

> ⚠️ **IMPORTANT**: Brume is currently in early development and is not yet ready for production use. Use at your own risk.

Brume is a modern cloud orchestration platform that allows you to create and manage your own personal cloud infrastructure. 

Choose a stack template, click deploy on your cloud account and enjoy an auto devops environments.

## Overview

Brume helps you create, deploy, and manage applications across your own infrastructure. Whether you're running applications on your own hardware, virtual machines, or a mix of cloud providers, Brume provides a unified interface to manage it all.

Find the right *stack* for your needs, and deploy it on your cloud. Brume takes care of the rest.

Key features:

- **Modular Stack architecture**: Anyone can create a stack, Brume only manages it
- **Application management**: Deploy and monitor your applications with ease
- **Infrastructure abstraction**: Manage different types of infrastructure through a consistent interface
- **Observability**: Built-in monitoring and logging
- **Developer-friendly**: Modern web console for intuitive management

## Components

Brume consists of several key components:

- **Orchestrator**: The central control plane that manages projects, deployments, and coordinates stacks on your differents account
- **Agent**: Used to communicate between Brume Orchestrator and your cloud account
- **Console**: Web-based user interface for managing your Brume environment
- **CLI**: Command-line interface for interacting with Brume

## Self Hosting

The easiest way to get started with Brume is using Docker Compose:

```bash
docker-compose up -d
```

This will start all necessary services, including:

- Brume Orchestrator
- Web Console
- Supporting services (PostgreSQL, Redis, ClickHouse)

Once everything is running, you can access:

- Web Console: http://localhost:3000

## Development

See the [development documentation](./docs/development.md) for information on setting up a development environment.

## Documentation

For detailed documentation, please see the [docs](./docs) directory.
