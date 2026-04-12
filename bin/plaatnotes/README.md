# PlaatNotes

A self-hosted note taking web app with rich markdown support

## Features

- Simple self-hosted in lightweight Docker container
- Rich markdown editor support
- Notes search, reordering and archiving
- Multiple users support with user management
- Google Keep Takeout import support

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
