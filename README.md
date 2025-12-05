# Brume

> ⚠️ **IMPORTANT**: Brume is currently in early development and is not yet ready for production use. Use at your own risk.

Brume is a modern cloud orchestration platform that allows you to create and manage your own cloud infrastructure without the hassle of using IaC software.

Choose a stack template, connect a cloud account (AWS for the moment) and click deploy and enjoy an auto devops environments.

## Overview

Brume helps you create, deploy, and manage applications across your own infrastructure. The goal is to remove as much as possible devops as possible, while keeping all the compute, data and networking on your cloud account.

Find the right *stack* for your needs, and deploy it on your cloud. Brume takes care of the rest.

Key features:

- **Modular Stack architecture**: Anyone can create a stack, Brume only manages it
- **Application management**: Deploy and monitor your applications with ease
- **Infrastructure abstraction**: Manage different types of infrastructure through a consistent interface (stacks, runners and builder)
- **Observability**: Built-in monitoring and logging (not yet available)
- **Developer-friendly**: Modern web console for intuitive management

## Components

Brume consists of several key components:

- **Orchestrator**: The central control plane that manages projects, deployments, and coordinates stacks on your differents account
- **Agent**: Used to communicate between Brume Orchestrator and your cloud account / deployed stacks
- **Console**: Web-based user interface for managing your Brume environment
- **CLI**: Command-line interface for interacting with Brume

## Trying it out

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

## Note

This is still a side project, and it is not backed by any kind of corporation. I try to make as professional as I can, but this is still a one person operation
