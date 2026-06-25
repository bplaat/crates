/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

PetiteVue.createApp({
    // Search state
    searchQuery: '',
    searchResults: [], // mixed [{ type:'artist'|'album', ...fields }]
    drilledArtist: null, // artist being drilled into
    artistAlbums: [], // albums shown when drilling
    isSearching: false,
    hasSearched: false,

    // Download state - plain objects, NOT Set/Map (petite-vue lacks collection handlers)
    tracks: {}, // { [globalIndex]: { label, status, percent, searchStart } }
    queuedAlbums: [], // { id, title, coverSmall, artistName, startIndex, trackCount }
    queuedIds: {}, // { [albumId]: true }
    now: Date.now(),

    init() {
        window.ipc.addEventListener('message', (e) => this.handlePush(JSON.parse(e.data)));
        setInterval(() => {
            this.now = Date.now();
        }, 1000);
    },

    handlePush(msg) {
        switch (msg.type) {
            case 'searchResults':
                this.searchResults = msg.results;
                this.isSearching = false;
                break;
            case 'artistAlbums':
                if (this.drilledArtist && this.drilledArtist.id === msg.artistId) {
                    this.artistAlbums = msg.albums;
                    this.isSearching = false;
                }
                break;
            case 'albumQueued': {
                this.queuedIds[msg.albumId] = true;
                this.queuedAlbums.push({
                    id: msg.albumId,
                    title: msg.title,
                    coverSmall: msg.coverSmall,
                    artistName: msg.artistName,
                    startIndex: msg.startIndex,
                    trackCount: msg.trackCount,
                });
                break;
            }
            case 'trackAdded':
                this.tracks[msg.index] = {
                    label: msg.label,
                    status: 'queued',
                    percent: 0,
                    searchStart: 0,
                };
                break;
            case 'trackSearching':
                if (this.tracks[msg.index]) {
                    this.tracks[msg.index].status = 'searching';
                    this.tracks[msg.index].searchStart = this.now;
                }
                break;
            case 'trackDownloading':
                if (this.tracks[msg.index]) {
                    this.tracks[msg.index].status = 'downloading';
                    this.tracks[msg.index].percent = msg.percent;
                }
                break;
            case 'trackWritingMetadata':
                if (this.tracks[msg.index]) {
                    this.tracks[msg.index].status = 'writingMetadata';
                }
                break;
            case 'trackDone':
                if (this.tracks[msg.index]) {
                    this.tracks[msg.index].status = 'done';
                }
                break;
            case 'trackFailed':
                if (this.tracks[msg.index]) {
                    this.tracks[msg.index].status = 'failed';
                }
                break;
        }
    },

    async search() {
        if (!this.searchQuery.trim()) return;
        this.isSearching = true;
        this.hasSearched = true;
        this.drilledArtist = null;
        this.artistAlbums = [];
        window.ipc.postMessage(JSON.stringify({ type: 'search', query: this.searchQuery }));
    },

    selectArtist(artist) {
        this.drilledArtist = artist;
        this.artistAlbums = [];
        this.isSearching = true;
        window.ipc.postMessage(JSON.stringify({ type: 'getArtistAlbums', artist_id: artist.id }));
    },

    clearArtist() {
        this.drilledArtist = null;
        this.artistAlbums = [];
    },

    visibleResults() {
        return this.drilledArtist ? this.artistAlbums : this.searchResults;
    },

    isQueued(albumId) {
        return !!this.queuedIds[albumId];
    },

    queueAlbum(album) {
        const albumId = album.id;
        if (this.queuedIds[albumId]) return;
        // Mark immediately so the button shows "Queued" right away
        this.queuedIds[albumId] = true;
        window.ipc.postMessage(JSON.stringify({ type: 'queueAlbum', album_id: albumId, with_cover: false }));
    },

    queueArtistAlbums(artist) {
        window.ipc.postMessage(JSON.stringify({ type: 'queueArtistAlbums', artist_id: artist.id, with_cover: false }));
    },

    albumTracks(album) {
        const result = [];
        for (let i = 0; i < album.trackCount; i++) {
            result.push(this.tracks[album.startIndex + i] || null);
        }
        return result;
    },

    albumDoneCount(album) {
        let count = 0;
        for (let i = 0; i < album.trackCount; i++) {
            const t = this.tracks[album.startIndex + i];
            if (t && (t.status === 'done' || t.status === 'failed')) count++;
        }
        return count;
    },

    albumSuccessCount(album) {
        let count = 0;
        for (let i = 0; i < album.trackCount; i++) {
            const t = this.tracks[album.startIndex + i];
            if (t && t.status === 'done') count++;
        }
        return count;
    },

    isAlbumDone(album) {
        if (album.trackCount === 0) return false;
        for (let i = 0; i < album.trackCount; i++) {
            const t = this.tracks[album.startIndex + i];
            if (!t || (t.status !== 'done' && t.status !== 'failed')) return false;
        }
        return true;
    },

    doneAlbums() {
        return this.queuedAlbums.filter((a) => this.isAlbumDone(a));
    },

    activeAlbums() {
        return this.queuedAlbums.filter((a) => !this.isAlbumDone(a));
    },

    trackTitle(label) {
        const dash = label.indexOf(' - ');
        return dash !== -1 ? label.slice(dash + 3) : label;
    },
}).mount('#app');
