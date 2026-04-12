# PlaatDeploy

A minimal self-hosted deployment service - a lightweight alternative to Dokploy/Coolify

## Features

- Simple self-hosted in a single Docker container
- GitHub App integration: auto-deploy on every push to main after CI passes
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

| Variable         | Required | Description                                          |
| ---------------- | -------- | ---------------------------------------------------- |
| `SERVER_ORIGIN`  | Yes      | Root origin (e.g. `https://apps.bplaat.nl`)          |
| `SERVER_PORT`    | No       | HTTP listen port (default: `8080`)                   |
| `DATABASE_PATH`  | No       | SQLite database path (default: `database.db`)        |
| `DEPLOY_PATH`    | No       | Directory for cloned repos (default: `deploy`)       |
| `ADMIN_EMAIL`    | No       | First-run admin email (default: `admin@example.com`) |
| `ADMIN_PASSWORD` | No       | First-run admin password (default: `Password123!`)   |

## License

Copyright &copy; 2026 [Bastiaan van der Plaat](https://bplaat.nl/)

Licensed under the [MIT](../../LICENSE) license.
