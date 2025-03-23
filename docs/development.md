# Brume Development Guide

This guide provides instructions for setting up a development environment for Brume and contributing to the project.

## Prerequisites

Before you begin, ensure you have the following installed:

- [Go](https://golang.org/doc/install) 1.22+
- [Node.js](https://nodejs.org/) 18+
- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)
- [git](https://git-scm.com/downloads)

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/your-org/brume.git
cd brume
```

### Environment Setup

Copy the example environment file:

```bash
cp .env.example .env
```

Edit the `.env` file as needed for your local environment.

### Starting the Development Environment

Brume uses Docker Compose to manage its development environment. To start all services:

```bash
docker-compose up -d
```

This will start:

- PostgreSQL
- Redis
- Temporal
- ClickHouse
- and other supporting services

### Running the Orchestrator

In development mode, you can run the orchestrator directly:

```bash
cd brume
go run cmd/brume/main.go master
```

Alternatively, you can use the Docker Compose service:

```bash
docker-compose up orchestrator
```

### Running the Agent

To run the agent locally:

```bash
cd agent
go run cmd/main.go
```

Or via Docker Compose:

```bash
docker-compose up agent
```

### Running the Console

The frontend console can be run in development mode:

```bash
cd console
yarn install
yarn dev
```

Or via Docker Compose:

```bash
docker-compose up console
```

## Project Structure

The Brume codebase is organized into several main directories:

- `brume/` - Core orchestration logic

  - `cmd/` - Entry points for binaries
  - `internal/` - Internal packages
  - `project/` - Project management
  - `deployment/` - Deployment logic
  - `machine/` - Machine management
  - `service/` - Service definitions

- `agent/` - Agent implementation

  - `cmd/` - Main entry point
  - `internal/` - Agent-specific internal packages
  - `runner/` - Task runners

- `console/` - Web UI

  - `src/` - React application source

- `infra/` - Infrastructure configuration and deployment

## Development Workflows

### Making Changes to the Orchestrator

1. Make your code changes
2. Run tests: `cd brume && go test ./...`
3. Run the orchestrator: `go run cmd/brume/main.go master`

### Making Changes to the Agent

1. Make your code changes
2. Run tests: `cd agent && go test ./...`
3. Run the agent: `go run cmd/main.go`

### Making Changes to the Console

1. Make your UI changes
2. The development server automatically reloads

## Testing

### Running Tests

To run Go tests:

```bash
cd brume
go test ./...

cd agent
go test ./...
```

To run frontend tests:

```bash
cd console
yarn test
```

### Integration Testing

Integration tests validate the interaction between components:

```bash
make integration-tests
```

## Code Generation

Brume uses code generation for GraphQL, Protocol Buffers, and other components.

### GraphQL Schema

After modifying the GraphQL schema:

```bash
cd brume
make public-gql
```

### Protocol Buffers

After changing the protobuf definitions:

```bash
make proto
```

## Debugging

### Orchestrator and Agent

You can use Delve for debugging Go code:

```bash
go install github.com/go-delve/delve/cmd/dlv@latest
dlv debug ./cmd/brume/main.go -- master
```

### Console

Use your browser's developer tools for debugging the frontend.

## Documentation

Please update documentation when making significant changes.

- README.md - Main project documentation
- docs/ - Detailed documentation
- Inline code comments

## Contribution Guidelines

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write or update tests
5. Update documentation
6. Submit a pull request

We follow standard Go code style guidelines. Please run `go fmt` and `golint` before submitting changes.
