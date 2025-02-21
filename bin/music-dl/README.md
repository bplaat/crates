# BassieBAS's Album Downloader
A Python script / tool that downloads complete albums via yt-dlp with the right metadata from the Deezer API

## Installation
You need to install [yt-dlp](https://github.com/yt-dlp/yt-dlp#installation) and [ffmpeg](https://ffmpeg.org/download.html) on your system and add its binaries to your path

You need to install the `mutagen` Python package:

```
pip install mutagen
```

## Usage
You can search and list album info with the `--list` argument:

```
./dl.py "Ordinary Songs 3" --list
```

Which returns the following output:

```
# Ordinary Songs 3 by Snail's House
Released at 2017-06-15 with 5 tracks
1. Good Day (2:36) by Snail's House
2. Bouquet (2:28) by Snail's House
3. Aloha (3:04) by Snail's House
4. あめあがりのうた (2:37) by Snail's House
5. Lullaby (3:31) by Snail's House
```

You can download an album without the `--list` argument and with the `--cover` argument you can also download the cover:

```
./dl.py "Ordinary Songs 3"
```

You can download all albums and EP's from a artist by using the `--artist` argument, you could use the `--singles` option to download also all its singles:

```
./dl.py "Snails House" --artist
```
