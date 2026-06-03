# PlaatNotes

A self-hosted note taking web app with rich markdown support

## Features

- Simple self-hosted in lightweight Docker container
- Rich markdown editor support
- Notes search, reordering and archiving
- Multiple users support with user management
- Google Keep Takeout import support

## IP Geolocation Database

PlaatNotes resolves visitor IPs to city/country using a local [DB-IP City Lite](https://db-ip.com/db/download/ip-to-city-lite) database, with ipinfo.io as fallback when the database is unavailable.

On first startup the app automatically downloads the current month's DB-IP City Lite MMDB file from `download.db-ip.com` and stores it in the data directory (`DATA_PATH`). If the download fails, it transparently falls back to ipinfo.io for every login request.

## Docker image

Example command to build the Docker image for PlaatNotes locally (from the root of the repository):

```sh
docker build --platform linux/amd64,linux/arm64 --tag ghcr.io/bplaat/plaatnotes:latest --file bin/plaatnotes/Dockerfile .
```

## Screenshot

![PlaatNotes Screenshot](docs/images/screenshot.png)

## License

Copyright © 2025-2026 [Bastiaan van der Plaat](https://bplaat.nl/)

Licensed under the [MIT](../../LICENSE) license.
