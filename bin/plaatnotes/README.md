# PlaatNotes

A self-hosted note taking web app with rich markdown support

## Features

- Simple self-hosted in lightweight Docker container
- Rich markdown editor support
- Notes search, reordering and archiving
- Multiple users support with user management
- Google Keep Takeout import support

## IP Geolocation Database (optional)

PlaatNotes resolves visitor IPs to city/country using either a local [DB-IP City Lite](https://db-ip.com/db/download/ip-to-city-lite) database or ipinfo.io as fallback.

The image exposes a `/data` volume (also used for the SQLite database).
Place the mmdb file there as `dbip-city-lite.mmdb` and the app picks it up automatically.

To use the local database:

1. Download the MMDB file (select **MMDB** format) from the link above.
2. Decompress: `gzip -d dbip-city-lite-*.mmdb.gz`
3. Copy into the data volume.

## Docker image

Example command to build the Docker image for PlaatNotes locally (from the root of the repository):

```sh
docker build --platform linux/amd64,linux/arm64 --tag ghcr.io/bplaat/plaatnotes:latest --file bin/plaatnotes/Dockerfile .
```

## Screenshot

![PlaatNotes Screenshot](docs/images/screenshot.png)

## License

Copyright &copy; 2025-2026 [Bastiaan van der Plaat](https://bplaat.nl/)

Licensed under the [MIT](../../LICENSE) license.
