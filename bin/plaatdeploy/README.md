# PlaatDeploy

A minimal self-hosted deployment service - a lightweight alternative to Dokploy/Coolify

## Features

- Simple self-hosted in a single Docker container
- Fine-grained GitHub personal access token integration: auto-deploy on every push to main after CI passes
- Docker and static site deployment (detected automatically)
- Reverse proxy with vhost routing (`*.yourdomain.com` per project)
- Multi-team support with user management

## Build types

Build type detection (inside `base_dir` of the repo):

- `index.html` present -> **Static** (served directly from disk)
- `Dockerfile` present -> **Docker** (built and run as container)

## Docker image

Build the Docker image from the **root of the repository**:

```sh
docker build --platform linux/amd64,linux/arm64 --tag ghcr.io/bplaat/plaatdeploy:latest --file bin/plaatdeploy/Dockerfile .
```

## Configuration

All configuration is via environment variables:

| Variable         | Required | Description                                                              |
| ---------------- | -------- | ------------------------------------------------------------------------ |
| `SERVER_ORIGIN`  | No\*     | Root origin (e.g. `https://apps.bplaat.nl`, default: `http://localhost`) |
| `SERVER_DEPLOYMENTS_ORIGIN` | No | Deployment wildcard origin (e.g. `https://*.bplaat.nl`, default: `https://*.{SERVER_ORIGIN host}`) |
| `SERVER_PORT`    | No       | HTTP listen port (default: `8080`)                                       |
| `DATA_PATH`      | No       | Data directory for `database.db`, `projects/`, and the geo-IP database (default: `.`) |
| `ADMIN_EMAIL`    | No       | First-run admin email (default: `admin@example.com`)                     |
| `ADMIN_PASSWORD` | No       | First-run admin password (default: `Password123!`)                       |

\* `SERVER_ORIGIN` has a default but should always be set in production: it drives
admin routing and the URLs used for GitHub webhooks. Set `SERVER_DEPLOYMENTS_ORIGIN` when
project subdomains are served through a different reverse-proxy domain.

## Security / trust model

- The container runs as `root` and bind-mounts the host Docker socket
  (`/var/run/docker.sock`). This is required to build and run project containers, but
  means PlaatDeploy has root-equivalent control of the host. Run it only on a trusted host.
- PlaatDeploy must be the TLS-terminating edge (or sit behind a proxy on a trusted
  network). Login rate limiting and session IP/geolocation use the client's socket
  address; `X-Forwarded-For` from inbound requests is intentionally **not** trusted, so a
  reverse proxy in front would collapse all clients to the proxy's address.

## License

Copyright &copy; 2026 [Bastiaan van der Plaat](https://bplaat.nl/)

Licensed under the [MIT](../../LICENSE) license.
