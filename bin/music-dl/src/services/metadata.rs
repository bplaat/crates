/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use percent_encoding::utf8_percent_encode;
use small_http::Request;

use crate::structs::deezer::{Album, AlbumList, AlbumSmall, ArtistList, ArtistSmall, Track};

const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0";

pub(crate) struct MetadataService;

impl MetadataService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn search_album(&self, query: &str) -> Result<Vec<AlbumSmall>> {
        Ok(Request::with_url(format!(
            "http://api.deezer.com/search/album?q={}",
            utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
        ))
        .header("User-Agent", USER_AGENT)
        .fetch()?
        .into_json::<AlbumList>()?
        .data)
    }

    pub(crate) fn seach_artist(&self, query: &str) -> Result<Vec<ArtistSmall>> {
        Ok(Request::with_url(format!(
            "http://api.deezer.com/search/artist?q={}",
            utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
        ))
        .header("User-Agent", USER_AGENT)
        .fetch()?
        .into_json::<ArtistList>()?
        .data)
    }

    pub(crate) fn get_artist_albums(&self, artist_id: i64) -> Result<Vec<AlbumSmall>> {
        Ok(
            Request::with_url(format!("http://api.deezer.com/artist/{}/albums", artist_id))
                .header("User-Agent", USER_AGENT)
                .fetch()?
                .into_json::<AlbumList>()?
                .data,
        )
    }

    pub(crate) fn get_album(&self, album_id: i64) -> Result<Album> {
        Ok(
            Request::with_url(format!("http://api.deezer.com/album/{}", album_id))
                .header("User-Agent", USER_AGENT)
                .fetch()?
                .into_json::<Album>()?,
        )
    }

    pub(crate) fn get_track(&self, track_id: i64) -> Result<Track> {
        Ok(
            Request::with_url(format!("http://api.deezer.com/track/{}", track_id))
                .header("User-Agent", USER_AGENT)
                .fetch()?
                .into_json::<Track>()?,
        )
    }

    pub(crate) fn download(&self, cover_url: &str) -> Result<Vec<u8>> {
        Ok(Request::with_url(cover_url)
            .header("User-Agent", USER_AGENT)
            .fetch()?
            .body)
    }
}
