/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use tiny_webview::{LogicalSize, Webview, WebviewBuilder};

use crate::services::metadata::MetadataService;
use crate::structs::deezer::{AlbumSmall, ArtistSmall};

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
enum IpcMessage {
    #[serde(rename = "search")]
    Search { query: String },
    #[serde(rename = "search-response")]
    SearchResponse {
        albums: Vec<AlbumSmall>,
        artists: Vec<ArtistSmall>,
    },
}

pub(crate) fn subcommmand_gui() -> ! {
    let mut webview = WebviewBuilder::new()
        .title("Music Downloader")
        .size(LogicalSize::new(1280.0, 720.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state(true)
        .force_dark_mode(true)
        .load_rust_embed::<WebAssets>()
        .build();

    webview.run(move |webview, event| {
        let metadata_service = MetadataService::new();
        if let tiny_webview::Event::PageMessageReceived(message) = event {
            match serde_json::from_str::<IpcMessage>(&message).expect("Can't parse message") {
                IpcMessage::Search { query } => {
                    let response = IpcMessage::SearchResponse {
                        albums: metadata_service
                            .search_albums(&query)
                            .expect(" Failed to search albums"),
                        artists: metadata_service
                            .search_artists(&query)
                            .expect("Failed to search artists"),
                    };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                _ => {}
            }
        }
    })
}
