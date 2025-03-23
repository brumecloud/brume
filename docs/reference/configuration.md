# Configuration Reference

This document provides a comprehensive reference for Brume configuration options.

## Configuration File Format

Brume uses TOML as its primary configuration format. Configuration files typically have a `.toml` extension.

Example:

```toml
# Example configuration file structure
[section]
key = "value"
```

## Main Configuration Sections

### Server Configuration

```toml
[server]
host = "0.0.0.0"
orchestrator_port = 9876
graphql_port = 9877
grpc_port = 9878
```

| Option              | Description                           | Default     |
| ------------------- | ------------------------------------- | ----------- |
| `host`              | The host address to bind to           | `"0.0.0.0"` |
| `orchestrator_port` | The port for the orchestrator service | `9876`      |
| `graphql_port`      | The port for the GraphQL API          | `9877`      |
| `grpc_port`         | The port for the gRPC API             | `9878`      |

### Database Configuration

```toml
[postgres]
host = "localhost"
port = 5432
db = "brume"
user = "brume"
password = "password"
max_idle = 10
max_open = 50
```

| Option     | Description                        | Default       |
| ---------- | ---------------------------------- | ------------- |
| `host`     | PostgreSQL host                    | `"localhost"` |
| `port`     | PostgreSQL port                    | `5432`        |
| `db`       | Database name                      | `"brume"`     |
| `user`     | Database username                  | `"brume"`     |
| `password` | Database password                  | -             |
| `max_idle` | Maximum number of idle connections | `10`          |
| `max_open` | Maximum number of open connections | `50`          |

### Redis Configuration

```toml
[redis]
host = "localhost"
port = 6379
db = 0
password = ""
```

| Option     | Description          | Default       |
| ---------- | -------------------- | ------------- |
| `host`     | Redis host           | `"localhost"` |
| `port`     | Redis port           | `6379`        |
| `db`       | Redis database index | `0`           |
| `password` | Redis password       | `""`          |

### Temporal Configuration

```toml
[temporal]
host = "localhost"
port = 7233
```

| Option | Description   | Default       |
| ------ | ------------- | ------------- |
| `host` | Temporal host | `"localhost"` |
| `port` | Temporal port | `7233`        |

### ClickHouse Configuration

```toml
[clickhouse]
host = "localhost"
port = 9000
db = "brume"
user = "default"
password = ""
```

| Option     | Description       | Default       |
| ---------- | ----------------- | ------------- |
| `host`     | ClickHouse host   | `"localhost"` |
| `port`     | ClickHouse port   | `9000`        |
| `db`       | Database name     | `"brume"`     |
| `user`     | Database username | `"default"`   |
| `password` | Database password | `""`          |

### Logging Configuration

```toml
[log]
level = "info"
db_level = "warn"
```

| Option     | Description            | Default  |
| ---------- | ---------------------- | -------- |
| `level`    | General logging level  | `"info"` |
| `db_level` | Database logging level | `"warn"` |

## Environment Variables

Brume also supports configuration through environment variables. Each configuration option can be set using an environment variable with the format `BRUME_SECTION_KEY`.

For example:

- `BRUME_SERVER_HOST` corresponds to `[server] host`
- `BRUME_POSTGRES_PASSWORD` corresponds to `[postgres] password`
