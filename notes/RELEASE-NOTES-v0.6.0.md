# Release v0.6.0 -- Deployment Strategy

## Summary

Phase 6 adds a comprehensive deployment strategy for greport. This release includes Docker Compose support for all three components (API, Dashboard, PostgreSQL), Kubernetes manifests, standalone VM deployment configs, a continuous deployment pipeline, and a full deployment guide. No application code changes are included -- only infrastructure, configuration, and documentation.

## What's New

### Dashboard Docker Support

- Added `docker/Dockerfile.dashboard` with a multi-stage build using Next.js standalone output
- Enabled `output: "standalone"` in `next.config.ts` for optimized Docker images (~150MB)
- `NEXT_PUBLIC_API_URL` is configurable as a build argument

### Docker Compose

- Added dashboard service to `docker/docker-compose.yml` with health check dependency on the API
- Added `GITHUB_TOKEN` and `RUST_LOG` environment variable pass-through from host
- Created `docker/docker-compose.prod.yml` production overlay with:
  - CPU and memory resource limits for all services
  - Required `DB_PASSWORD` (fails if unset)
  - Removed external PostgreSQL port exposure
  - JSON file logging with rotation
  - Restart policy set to `always`
- Created `docker/.env.example` with all configurable environment variables documented

### Kubernetes

Created a full set of manifests in `k8s/`:

| Manifest | Resources |
|----------|-----------|
| `namespace.yaml` | `greport` namespace |
| `secrets.yaml` | Template for github-token, db-password, database-url |
| `configmap.yaml` | API configuration (log level, rate limits, cache TTL, SLA thresholds) |
| `postgres.yaml` | StatefulSet with headless Service, 10Gi PVC, liveness/readiness probes |
| `api.yaml` | Deployment (2 replicas), Service, HPA (2-8 pods on CPU/memory), init container for DB readiness |
| `dashboard.yaml` | Deployment (2 replicas), Service |
| `ingress.yaml` | Nginx Ingress routing `/api` to API, `/` to Dashboard, TLS-ready |

### Standalone (VM) Deployment

- `deploy/greport-api.service` -- systemd unit with security hardening (NoNewPrivileges, ProtectSystem, PrivateTmp)
- `deploy/nginx-greport.conf` -- Nginx reverse proxy config with WebSocket support and commented TLS block

### Continuous Deployment Pipeline

- Created `.github/workflows/deploy.yml`:
  - Triggered by version tags (`v*`) or manual workflow dispatch
  - Builds and pushes API, Dashboard, and CLI images to ghcr.io
  - Optional Kubernetes deployment with environment selection (staging/production)
  - Rollout verification and pod status checks
- Updated `.github/workflows/ci.yml`:
  - Added `phase6` to branch triggers
  - Added dashboard Docker image build to the Docker job
- Updated `.github/workflows/release.yml`:
  - Added dashboard image build and push to ghcr.io

### Deployment Guide

Created `docs/deployment-guide.md` covering:

1. Architecture overview
2. Prerequisites per deployment model
3. Complete configuration reference (16 environment variables)
4. Docker Compose quick start and production mode
5. Standalone VM setup (PostgreSQL, systemd, Nginx, TLS)
6. Kubernetes manifest apply order, secrets, ingress, scaling, external DB
7. CD pipeline usage, tag-based flow, rollback procedures
8. Database management (auto-migrations, manual migration, backup/restore)
9. Monitoring (health endpoint, Docker/K8s probes, log monitoring)
10. Troubleshooting common issues

## Bug Fixes

- Fixed Rust Docker base image from `rust:1.88-slim` to `rust:1.87-slim` in `Dockerfile.api` and `Dockerfile.cli`

## Files Changed

### Modified (6)
- `docker/Dockerfile.api` -- Rust image version fix
- `docker/Dockerfile.cli` -- Rust image version fix
- `dashboard/next.config.ts` -- Added standalone output
- `docker/docker-compose.yml` -- Added dashboard service and environment variables
- `.github/workflows/ci.yml` -- Added phase6 trigger and dashboard Docker build
- `.github/workflows/release.yml` -- Added dashboard image build and push

### Added (14)
- `docker/Dockerfile.dashboard`
- `docker/docker-compose.prod.yml`
- `docker/.env.example`
- `k8s/namespace.yaml`
- `k8s/secrets.yaml`
- `k8s/configmap.yaml`
- `k8s/postgres.yaml`
- `k8s/api.yaml`
- `k8s/dashboard.yaml`
- `k8s/ingress.yaml`
- `deploy/greport-api.service`
- `deploy/nginx-greport.conf`
- `.github/workflows/deploy.yml`
- `docs/deployment-guide.md`

## Deployment Models

| Model | Use Case |
|-------|----------|
| Docker Compose | Local development, small teams, single-server deployments |
| Docker Compose + prod overlay | Production single-server with resource limits and log rotation |
| Standalone (VM) | Bare-metal or VM deployments without containers |
| Kubernetes | Scalable production deployments with auto-scaling and rolling updates |

## Docker Images

All images are published to GitHub Container Registry:

| Image | Path |
|-------|------|
| API | `ghcr.io/<owner>/<repo>/greport-api` |
| Dashboard | `ghcr.io/<owner>/<repo>/greport-dashboard` |
| CLI | `ghcr.io/<owner>/<repo>/greport` |

## Upgrade Notes

- No breaking changes to application behavior
- Existing Docker Compose users: the `docker-compose.yml` now includes a dashboard service and uses `${GITHUB_TOKEN}` from the host environment. Set the variable or create a `.env` file from the provided template.
- The `DB_PASSWORD` defaults to `greport` in the base compose file. For production, use the prod overlay which requires an explicit password.
