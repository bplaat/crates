/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::sync::{Arc, mpsc};
use std::thread;

use bwebview::{
    Event, EventLoopBuilder, EventLoopProxy, LogicalSize, WebviewBuilder, WebviewEvent,
    WindowBuilder,
};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};

use crate::args::{Args, Subcommand};
use crate::downloader::{Downloader, ProgressEvent};
use crate::services::metadata::MetadataService;
use crate::structs::deezer::AlbumSmall;

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

// MARK: IPC messages

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum IpcRequest {
    Search { query: String },
    GetArtistAlbums { artist_id: i64 },
    QueueAlbum { album_id: i64, with_cover: bool },
    QueueArtistAlbums { artist_id: i64, with_cover: bool },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
enum IpcPush {
    SearchResults {
        results: Vec<serde_json::Value>,
    },
    ArtistAlbums {
        #[serde(rename = "artistId")]
        artist_id: i64,
        albums: Vec<AlbumSmall>,
    },
    AlbumQueued {
        #[serde(rename = "albumId")]
        album_id: i64,
        title: String,
        #[serde(rename = "coverSmall")]
        cover_small: String,
        #[serde(rename = "artistName")]
        artist_name: String,
        #[serde(rename = "startIndex")]
        start_index: usize,
        #[serde(rename = "trackCount")]
        track_count: usize,
    },
    TrackAdded {
        index: usize,
        label: String,
    },
    TrackSearching {
        index: usize,
    },
    TrackDownloading {
        index: usize,
        percent: f32,
    },
    TrackWritingMetadata {
        index: usize,
    },
    TrackDone {
        index: usize,
    },
    TrackFailed {
        index: usize,
    },
}

impl From<ProgressEvent> for Option<IpcPush> {
    fn from(event: ProgressEvent) -> Self {
        Some(match event {
            ProgressEvent::AlbumQueued {
                album_id,
                title,
                cover_small,
                artist_name,
                start_index,
                track_count,
            } => IpcPush::AlbumQueued {
                album_id,
                title,
                cover_small,
                artist_name,
                start_index,
                track_count,
            },
            ProgressEvent::Added { index, label } => IpcPush::TrackAdded { index, label },
            ProgressEvent::Searching { index } => IpcPush::TrackSearching { index },
            ProgressEvent::Downloading { index, percent } => {
                IpcPush::TrackDownloading { index, percent }
            }
            ProgressEvent::WritingMetadata { index } => IpcPush::TrackWritingMetadata { index },
            ProgressEvent::Done { index } => IpcPush::TrackDone { index },
            ProgressEvent::Failed { index } => IpcPush::TrackFailed { index },
        })
    }
}

// MARK: Worker

enum GuiCommand {
    Search { query: String },
    GetArtistAlbums { artist_id: i64 },
    QueueAlbum { album_id: i64, with_cover: bool },
    QueueArtistAlbums { artist_id: i64, with_cover: bool },
}

fn gui_args(with_cover: bool) -> Args {
    Args {
        subcommand: Subcommand::Download,
        with_cover,
        ..Args::default()
    }
}

fn send_push(proxy: &Arc<EventLoopProxy>, push: IpcPush) {
    let json = serde_json::to_string(&push).expect("Failed to serialize IPC push");
    proxy.send_user_event(json);
}

fn build_search_results(
    tracks: Vec<crate::structs::deezer::SearchTrack>,
) -> Vec<serde_json::Value> {
    let mut seen_artists: HashSet<i64> = HashSet::new();
    let mut seen_albums: HashSet<i64> = HashSet::new();
    let mut results = Vec::new();
    for track in tracks {
        if seen_artists.insert(track.artist.id) {
            results.push(serde_json::json!({
                "type": "artist",
                "id": track.artist.id,
                "name": track.artist.name,
                "picture_small": track.artist.picture_small,
            }));
        }
        if seen_albums.insert(track.album.id) {
            results.push(serde_json::json!({
                "type": "album",
                "id": track.album.id,
                "title": track.album.title,
                "cover": track.album.cover,
                "cover_small": track.album.cover_small,
                "artist_id": track.artist.id,
                "artist_name": track.artist.name,
            }));
        }
    }
    results
}

fn background_worker(cmd_rx: mpsc::Receiver<GuiCommand>, proxy: Arc<EventLoopProxy>) {
    let mut metadata_service = MetadataService::new();
    let mut downloader = Downloader::new();
    let (prog_tx, prog_rx) = mpsc::channel::<ProgressEvent>();

    let bridge_proxy = Arc::clone(&proxy);
    thread::spawn(move || {
        for event in prog_rx {
            if let Some(push) = Option::<IpcPush>::from(event) {
                let json = serde_json::to_string(&push).expect("Failed to serialize IPC push");
                bridge_proxy.send_user_event(json);
            }
        }
    });

    for cmd in cmd_rx {
        match cmd {
            GuiCommand::Search { query } => {
                let results = metadata_service
                    .search(&query)
                    .map(|list| build_search_results(list.data))
                    .unwrap_or_default();
                send_push(&proxy, IpcPush::SearchResults { results });
            }
            GuiCommand::GetArtistAlbums { artist_id } => {
                let albums = metadata_service
                    .get_artist_albums(artist_id)
                    .unwrap_or_default();
                send_push(&proxy, IpcPush::ArtistAlbums { artist_id, albums });
            }
            GuiCommand::QueueAlbum {
                album_id,
                with_cover,
            } => {
                let args = gui_args(with_cover);
                downloader
                    .queue_album(&args, &mut metadata_service, album_id, prog_tx.clone())
                    .ok();
            }
            GuiCommand::QueueArtistAlbums {
                artist_id,
                with_cover,
            } => {
                if let Ok(albums) = metadata_service.get_artist_albums(artist_id) {
                    let args = gui_args(with_cover);
                    for album in albums {
                        downloader
                            .queue_album(&args, &mut metadata_service, album.id, prog_tx.clone())
                            .ok();
                    }
                }
            }
        }
    }
}

// MARK: Entry point

pub(crate) fn run() {
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "MusicDL")
        .build();

    let proxy = Arc::new(event_loop.create_proxy());

    let (cmd_tx, cmd_rx) = mpsc::channel::<GuiCommand>();
    let worker_proxy = Arc::clone(&proxy);
    thread::spawn(move || background_worker(cmd_rx, worker_proxy));

    let mut window = WindowBuilder::new()
        .title("Music Downloader")
        .size(LogicalSize::new(1200.0, 720.0))
        .min_size(LogicalSize::new(900.0, 500.0))
        .center()
        .remember_window_state()
        .build();

    let mut webview = WebviewBuilder::new(&window)
        .load_rust_embed::<WebAssets>()
        .build();

    event_loop.run(move |event| match event {
        Event::UserEvent(json) => webview.send_ipc_message(json),
        Event::Webview(WebviewEvent::PageTitleChange(title)) => window.set_title(title),
        Event::Webview(WebviewEvent::MessageReceive(msg)) => {
            if let Ok(req) = serde_json::from_str::<IpcRequest>(&msg) {
                let cmd = match req {
                    IpcRequest::Search { query } => GuiCommand::Search { query },
                    IpcRequest::GetArtistAlbums { artist_id } => {
                        GuiCommand::GetArtistAlbums { artist_id }
                    }
                    IpcRequest::QueueAlbum {
                        album_id,
                        with_cover,
                    } => GuiCommand::QueueAlbum {
                        album_id,
                        with_cover,
                    },
                    IpcRequest::QueueArtistAlbums {
                        artist_id,
                        with_cover,
                    } => GuiCommand::QueueArtistAlbums {
                        artist_id,
                        with_cover,
                    },
                };
                cmd_tx.send(cmd).ok();
            }
        }
        _ => {}
    });
}
