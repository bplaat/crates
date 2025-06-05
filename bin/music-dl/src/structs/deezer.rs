/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(dead_code)]

use serde::Deserialize;

// Album
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AlbumList {
    pub(crate) data: Vec<AlbumSmall>,
    pub(crate) total: i64,
}

#[derive(Deserialize)]
pub(crate) struct AlbumSmall {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) cover: String,
    pub(crate) cover_small: Option<String>,
    pub(crate) cover_medium: Option<String>,
    pub(crate) cover_big: Option<String>,
    pub(crate) cover_xl: Option<String>,
    pub(crate) record_type: String,
    pub(crate) explicit_lyrics: bool,
    pub(crate) r#type: String,
}

#[derive(Deserialize, Clone)]
pub(crate) struct Album {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) cover: String,
    pub(crate) cover_small: Option<String>,
    pub(crate) cover_medium: Option<String>,
    pub(crate) cover_big: Option<String>,
    pub(crate) cover_xl: Option<String>,
    pub(crate) genres: GenreList,
    pub(crate) nb_tracks: i64,
    pub(crate) duration: i64,
    pub(crate) release_date: String,
    pub(crate) record_type: String,
    pub(crate) explicit_lyrics: bool,
    pub(crate) contributors: Vec<ArtistSmall>,
    pub(crate) r#type: String,
    pub(crate) tracks: TrackList,
}

// Artist
#[derive(Deserialize)]
pub(crate) struct ArtistList {
    pub(crate) data: Vec<ArtistSmall>,
    pub(crate) total: i64,
}

#[derive(Deserialize, Clone)]
pub(crate) struct ArtistSmall {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) r#type: String,
}

#[derive(Deserialize)]
pub(crate) struct Artist {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) picture: String,
    pub(crate) picture_small: String,
    pub(crate) picture_medium: String,
    pub(crate) picture_big: String,
    pub(crate) picture_xl: String,
    pub(crate) r#type: String,
}

// Genre
#[derive(Deserialize, Clone)]
pub(crate) struct GenreList {
    pub(crate) data: Vec<Genre>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct Genre {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) r#type: String,
}

// Track
#[derive(Deserialize, Clone)]
pub(crate) struct TrackList {
    pub(crate) data: Vec<TrackSmall>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct TrackSmall {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) duration: i64,
    pub(crate) explicit_lyrics: bool,
    pub(crate) r#type: String,
}

#[derive(Deserialize)]
pub(crate) struct Track {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) duration: i64,
    pub(crate) track_position: i64,
    pub(crate) disk_number: i64,
    pub(crate) release_date: String,
    pub(crate) explicit_lyrics: bool,
    pub(crate) bpm: f64,
    pub(crate) contributors: Vec<ArtistSmall>,
    pub(crate) r#type: String,
}
