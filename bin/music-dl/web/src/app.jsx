/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';
import { send, request } from './ipc.js';

export function SearchColumn() {
    const [searchQuery, setSearchQuery] = useState('');
    const [albums, setAlbums] = useState([]);
    const [artists, setArtists] = useState([]);

    const search = async (event) => {
        event.preventDefault();
        if (searchQuery.trim() === '') {
            return;
        }
        const { albums, artists } = await request('search', { query: searchQuery });
        setAlbums(albums);
        setArtists(artists);
    };

    return (
        <div className="column">
            <h2>Search</h2>

            <form onSubmit={search}>
                <input
                    type="text"
                    placeholder="Search for albums or artists..."
                    value={searchQuery}
                    onInput={(e) => setSearchQuery(e.target.value)}
                />
                <button type="submit">Search</button>
            </form>

            <h3>Albums</h3>
            {albums.length > 0 ? (
                albums.map((album) => (
                    <div class="item" key={album.id}>
                        <img src={album.cover_big} alt={`${album.title} cover`} width="50" height="50" />
                        <strong>{album.title}</strong>
                    </div>
                ))
            ) : (
                <p>No albums found</p>
            )}

            <h3>Artists</h3>
            {artists.length > 0 ? (
                artists.map((artist) => (
                    <div class="item" key={artist.id}>
                        <img src={artist.picture_big} alt={`${artist.name} picture`} width="50" height="50" />
                        <strong>{artist.name}</strong>
                    </div>
                ))
            ) : (
                <p>No artists found</p>
            )}
        </div>
    );
}

export function DownloadQueueColumn() {
    return (
        <div className="column">
            <h2>Downloading</h2>
        </div>
    );
}

export function DoneColumn() {
    return (
        <div className="column">
            <h2>Ready</h2>
        </div>
    );
}

export function App() {
    return (
        <>
            <SearchColumn />
            <DownloadQueueColumn />
            <DoneColumn />
        </>
    );
}
