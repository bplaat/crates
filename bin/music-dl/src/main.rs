/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio, exit};
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use threadpool::ThreadPool;

use crate::args::{Args, Subcommand, parse_args};
use crate::services::metadata::MetadataService;
use crate::structs::deezer::{Album, Track};
use crate::structs::youtube::Video;
use crate::utils::{escape_path, get_album_ids};

mod args;
mod services;
mod structs;
mod utils;

const DOWNLOAD_THREAD_COUNT: usize = 16;
const TRACK_DURATION_SLACK: i64 = 5;

// MARK: Subcommands
fn subcommand_download(args: &Args) -> Result<()> {
    if args.query.is_empty() {
        eprintln!("Query argument is required");
        exit(1);
    }

    // Find album ids
    let metadata_service = MetadataService::new();
    let album_ids = get_album_ids(&metadata_service, args)?;

    // Start downloading albums
    let mut pool = ThreadPool::new(DOWNLOAD_THREAD_COUNT);
    for album_id in album_ids {
        download_album(args, &mut pool, metadata_service, album_id)?;
    }
    pool.join();
    Ok(())
}

fn download_album(
    args: &Args,
    pool: &mut ThreadPool,
    metadata_service: MetadataService,
    album_id: i64,
) -> Result<()> {
    // Download album metadata
    let album = metadata_service.get_album(album_id)?;

    // Download album cover
    let album_folder = format!(
        "{}/{}/{}",
        args.output_dir,
        escape_path(&album.contributors[0].name),
        escape_path(&album.title)
    );
    let album_cover = if let Some(album_cover_xl) = &album.cover_xl {
        let album_cover = metadata_service.download(album_cover_xl)?;
        if args.with_cover {
            fs::write(format!("{}/cover.jpg", album_folder), &album_cover)?;
        }
        Some(album_cover)
    } else {
        None
    };

    // Calculate total number of disks
    let mut tracks = Vec::new();
    let mut album_nb_disks = 0;
    let mut previous_disk_number = 0;
    for track in &album.tracks.data {
        let track = metadata_service.get_track(track.id)?;
        if track.disk_number != previous_disk_number {
            album_nb_disks += 1;
            previous_disk_number = track.disk_number;
        }
        tracks.push(track);
    }

    // Download tracks
    for (index, track) in tracks.into_iter().enumerate() {
        let album = album.clone();
        let album_folder = album_folder.clone();
        let album_cover = album_cover.clone();
        pool.execute(move || {
            let _ = download_track(
                album,
                album_folder,
                album_cover,
                album_nb_disks,
                track,
                index,
            );
        });
    }
    Ok(())
}

fn download_track(
    album: Album,
    album_folder: String,
    album_cover: Option<Vec<u8>>,
    album_nb_disks: i64,
    track: Track,
    track_index: usize,
) -> Result<()> {
    // Search correct YouTube video
    let search_queries = [
        format!("{} - {}", album.contributors[0].name, track.title),
        format!(
            "{} - {} - {}",
            album.contributors[0].name, album.title, track.title
        ),
        format!("{} - {}", album.title, track.title),
    ];
    for search_query in search_queries {
        println!("Searching {}...", search_query);
        let mut search_process = Command::new("yt-dlp")
            .arg("--dump-json")
            .arg(format!("ytsearch25:{}", search_query))
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = search_process
            .stdout
            .as_mut()
            .expect("Can't read from yt-dlp process");
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let video = serde_json::from_str::<Video>(&line?)?;

            if track.duration >= video.duration - TRACK_DURATION_SLACK
                && track.duration <= video.duration + TRACK_DURATION_SLACK
            {
                search_process.kill()?;

                // Download video
                let path = format!(
                    "{}/{} - {} - {:0track_index_width$} - {}.m4a",
                    album_folder,
                    escape_path(&album.contributors[0].name),
                    escape_path(&album.title),
                    track_index + 1,
                    escape_path(&track.title),
                    track_index_width = (album.nb_tracks as f64).log10().ceil() as usize
                );
                let mut download_process = Command::new("yt-dlp")
                    .arg("--newline")
                    .arg("-f")
                    .arg("bestaudio[ext=m4a]")
                    .arg(format!("https://www.youtube.com/watch?v={}", video.id))
                    .arg("-o")
                    .arg(&path)
                    .stdout(Stdio::piped())
                    .spawn()?;
                println!("Downloading {}...", path);
                download_process.wait()?;

                // Update metadata
                println!("Updating metadata {}...", path);
                let mut tag = mp4ameta::Tag::default();
                tag.set_title(&track.title);
                for artist in album.contributors.iter() {
                    tag.add_artist(artist.name.as_str());
                }
                for artist in track.contributors.iter() {
                    if album
                        .contributors
                        .iter()
                        .any(|album_artist| album_artist.id == artist.id)
                    {
                        continue;
                    }
                    tag.add_artist(artist.name.as_str());
                }
                tag.set_album(&album.title);
                for artist in album.contributors.iter() {
                    tag.add_album_artist(artist.name.as_str());
                }
                for genre in album.genres.data.iter() {
                    tag.add_genre(genre.name.as_str());
                }
                tag.set_track(track.track_position as u16, album.nb_tracks as u16);
                tag.set_disc(track.disk_number as u16, album_nb_disks as u16);
                tag.set_year(
                    album
                        .release_date
                        .split('-')
                        .next()
                        .expect("Can't parse track release year"),
                );
                tag.set_bpm(track.bpm as u16);
                if let Some(album_cover) = album_cover {
                    tag.set_artwork(mp4ameta::Img::jpeg(album_cover));
                }
                tag.write_to_path(path)?;

                return Ok(());
            }
        }
    }
    // FIXME: No video found for track
    Ok(())
}

fn subcommand_list(args: &Args) -> Result<()> {
    if args.query.is_empty() {
        eprintln!("Query argument is required");
        exit(1);
    }

    // Find album ids
    let metadata_service = MetadataService::new();
    let album_ids = get_album_ids(&metadata_service, args)?;

    // List albums
    for album_id in album_ids {
        let album = metadata_service.get_album(album_id)?;
        let mut tracks = Vec::new();
        let mut album_is_multi_disk = false;
        for track in &album.tracks.data {
            let track = metadata_service.get_track(track.id)?;
            if track.disk_number > 1 {
                album_is_multi_disk = true;
            }
            tracks.push(track);
        }

        println!(
            "# {} by {}\n",
            album.title,
            album
                .contributors
                .iter()
                .map(|artist| artist.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "**Released at {} with {} tracks**\n",
            album.release_date, album.nb_tracks
        );

        let mut last_disk_number = 0;
        for track in &tracks {
            if album_is_multi_disk && track.disk_number != last_disk_number {
                println!(
                    "{}## Disk {}\n",
                    if track.disk_number != 1 { "\n" } else { "" },
                    track.disk_number
                );
                last_disk_number = track.disk_number;
            }
            println!(
                "{}. {} ({}:{:02}) by {}",
                track.track_position,
                track.title,
                track.duration / 60,
                track.duration % 60,
                track
                    .contributors
                    .iter()
                    .map(|artist| artist.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        println!();

        // Sleep for 0.5s to avoid Deezer rate limiting
        sleep(Duration::from_millis(500));
    }
    Ok(())
}

fn subcommand_help() {
    println!(
        r"Usage: music-dl [SUBCOMMAND] [OPTIONS]

Options:
  -o <dir>            Change output directory
  -i, --id            Query is a Deezer ID
  -a, --artist        Query is an artist name
  -s, --with-singles  Include singles of artist
  -c, --with-cover    Also download cover image

Subcommands:
  download            Download album or artist
  list                List all albums of artist
  help                Print this help message
  version             Print the version number"
    );
}

fn subcommand_version() {
    println!("music-dl v{}", env!("CARGO_PKG_VERSION"));
}

// MARK: Main
fn main() -> Result<()> {
    let args = parse_args();
    match args.subcommand {
        Subcommand::Download => subcommand_download(&args)?,
        Subcommand::List => subcommand_list(&args)?,
        Subcommand::Help => subcommand_help(),
        Subcommand::Version => subcommand_version(),
    }
    Ok(())
}
