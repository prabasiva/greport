# greport

A command-line tool for GitHub repository analytics and reporting.

greport fetches data from the GitHub API and generates reports on issues, pull requests, releases, and contributors. It supports multiple output formats and can be configured to track SLA compliance.

## Installation

```bash
git clone https://github.com/prabasiva/greport.git
cd greport
cargo install --path crates/greport-cli
```

## Setup

Set your GitHub personal access token:

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
```

Or create a config file at `~/.config/greport/config.toml`:

```toml
[github]
token = "ghp_xxxxxxxxxxxx"

[defaults]
repo = "owner/repo"
format = "table"

[sla]
response_time_hours = 24
resolution_time_hours = 168

[sla.priority.critical]
response_time_hours = 4
resolution_time_hours = 24
```

## Usage

```bash
# Issues
greport issues list -r owner/repo
greport issues metrics -r owner/repo
greport issues velocity -r owner/repo --period week --last 12
greport issues stale -r owner/repo --days 30
greport issues sla -r owner/repo

# Pull requests
greport prs list -r owner/repo
greport prs metrics -r owner/repo

# Releases
greport releases list -r owner/repo
greport releases notes -r owner/repo --milestone "v1.0"

# Contributors
greport contrib list -r owner/repo
```

Output formats: `table` (default), `json`, `csv`, `markdown`

```bash
greport issues list -r owner/repo -f json
```

## Project Layout

```
crates/
  greport-core/   # Core library
  greport-cli/    # CLI tool
  greport-api/    # REST API server
  greport-db/     # Database layer
```

## Building

Requires Rust 1.87 or later.

```bash
cargo build --release
cargo test
```

## API Server

```bash
docker compose -f docker/docker-compose.yml up
```

The API runs at `http://localhost:3000/api/v1`

## License

MIT
