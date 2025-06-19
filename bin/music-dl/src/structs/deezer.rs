/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// Album
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AlbumList {
    pub data: Vec<AlbumSmall>,
    pub total: i64,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct AlbumSmall {
    pub id: i64,
    pub title: String,
    pub cover: String,
    pub cover_small: Option<String>,
    pub cover_medium: Option<String>,
    pub cover_big: Option<String>,
    pub cover_xl: Option<String>,
    pub record_type: String,
    pub explicit_lyrics: bool,
    pub r#type: String,
}

#[derive(Clone, Deserialize)]
pub(crate) struct Album {
    pub id: i64,
    pub title: String,
    pub cover: String,
    pub cover_small: Option<String>,
    pub cover_medium: Option<String>,
    pub cover_big: Option<String>,
    pub cover_xl: Option<String>,
    pub genres: GenreList,
    pub nb_tracks: i64,
    pub duration: i64,
    pub release_date: String,
    pub record_type: String,
    pub explicit_lyrics: bool,
    pub contributors: Vec<ArtistSmall>,
    pub r#type: String,
    pub tracks: TrackList,
}

// Artist
#[derive(Deserialize)]
pub(crate) struct ArtistList {
    pub data: Vec<ArtistSmall>,
    pub total: i64,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct ArtistSmall {
    pub id: i64,
    pub name: String,
    pub picture: String,
    pub picture_small: String,
    pub picture_medium: String,
    pub picture_big: String,
    pub picture_xl: String,
    pub r#type: String,
}

// Genre
#[derive(Deserialize, Clone)]
pub(crate) struct GenreList {
    pub data: Vec<Genre>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct Genre {
    pub id: i64,
    pub name: String,
    pub r#type: String,
}

// Track
#[derive(Clone, Deserialize)]
pub(crate) struct TrackList {
    pub data: Vec<TrackSmall>,
}

#[derive(Clone, Deserialize)]
pub(crate) struct TrackSmall {
    pub id: i64,
    pub title: String,
    pub duration: i64,
    pub explicit_lyrics: bool,
    pub r#type: String,
}

#[derive(Deserialize)]
pub(crate) struct Track {
    pub id: i64,
    pub title: String,
    pub duration: i64,
    pub track_position: i64,
    pub disk_number: i64,
    pub release_date: String,
    pub explicit_lyrics: bool,
    pub bpm: f64,
    pub contributors: Vec<ArtistSmall>,
    pub r#type: String,
}
