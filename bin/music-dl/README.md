# BassieBAS's Music Downloader

A tool that downloads complete albums from YouTube with the correct metadata from Deezer

## Installation

First you need to install [yt-dlp](https://github.com/yt-dlp/yt-dlp#installation) and [ffmpeg](https://ffmpeg.org/download.html), then download, compile and install with:

```sh
cargo install --git https://github.com/bplaat/crates.git music-dl
```

## Usage

You can search and list album info with the `list` subcommand:

```sh
music-dl list "Ordinary Songs 3"
```

Which returns the following output:

```md
# Ordinary Songs 3 by Snail's House

**Released at 2017-06-15 with 5 tracks**

1. Good Day (2:36) by Snail's House
2. Bouquet (2:28) by Snail's House
3. Aloha (3:04) by Snail's House
4. あめあがりのうた (2:37) by Snail's House
5. Lullaby (3:31) by Snail's House
```

You can download an album without the `download` subcommand, use the `--with-cover` argument to also download the Album cover:

```sh
music-dl download "Ordinary Songs 3"
```

You can download all albums and EP's from a artist by using the `--artist` argument, you could use the `--with-singles` argument to download also all its singles:

```
music-dl download --artist "Snails House"
```
