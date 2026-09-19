# Plaat Deploy

A small self-hosted deployment service for GitHub repositories.

## Features

- Static and Dockerfile deployments
- GitHub webhooks and deployment statuses
- Automatic deployment detection with manual overrides
- Traefik routing and persistent Docker volumes

## Setup

Create the shared network, configure `.env`, and start the stack:

```sh
docker network create plaatdeploy
docker compose up --build -d
```

Set `DEPLOYMENTS_DOMAIN=*.apps.example.com` and point that wildcard DNS record to the server. Projects
are served at `<project>.apps.example.com`. The `*.` prefix is required in the setting.

Set a strong `ADMIN_PASSWORD`; it is required when the database creates the initial account.
`ADMIN_EMAIL` defaults to `admin@example.com`. Add a fine-grained GitHub token with:

- Contents: read
- Webhooks: read and write
- Deployments: read and write

## Deployments

Static projects use Nginx Alpine. Dockerfile projects must expose a numeric port. Repository pushes
trigger redeployment, and Dockerfile `VOLUME` data persists between deployments.

Plaat Deploy, Traefik, and deployed projects use the external `plaatdeploy` network. Other Compose
stacks can join this network and use the same Traefik instance.

## Security

Plaat Deploy mounts the Docker socket and has root-equivalent host access. Use it only on a trusted
server and protect its data volume, which contains SQLite data and GitHub tokens.

## License

Copyright (c) 2026 Bastiaan van der Plaat

Licensed under the [MIT](../../LICENSE) license.
