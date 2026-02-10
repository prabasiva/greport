## What's New

### Multi-Organization Support (Phase 7)

- **Per-org tokens**: Configure multiple GitHub organizations with dedicated tokens via `[[organizations]]` in config.toml or `GREPORT_ORG_*_TOKEN` environment variables
- **Client registry**: Automatic token resolution based on repository owner with fallback to default token
- **CLI multi-repo execution**: Run commands across all configured org repos, or target a specific org with `--org`
- **CLI orgs commands**: `greport orgs list` and `greport orgs show <name>` for managing organizations
- **API org endpoints**: `GET /api/v1/orgs` and `GET /api/v1/orgs/{org}/repos`
- **Cross-org aggregation**: `GET /api/v1/aggregate/orgs/issues` and `GET /api/v1/aggregate/orgs/pulls`
- **Database migration**: `org_name` column on repositories table, new `organizations` table
- **Token validation**: Validates configured tokens on API startup and CLI `--verbose`
- **Backward compatible**: Existing single-token configurations continue to work unchanged

### Other Changes

- Add GHCR docker compose configuration
- Bump workspace version to 0.7.2
