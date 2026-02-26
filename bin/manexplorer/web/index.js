/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

if (navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh')) {
    document.body.classList.add('is-bwebview-macos');
}

window.addEventListener('contextmenu', (event) => event.preventDefault());

const pageNames = {
    1: 'User Commands',
    2: 'System Calls',
    3: 'C Library Functions',
    4: 'Devices and Special Files',
    5: 'File Formats and Conventions',
    6: 'Games et. Al.',
    7: 'Miscellanea',
    8: 'System Administration',
    9: 'Kernel Routines',
};

PetiteVue.createApp({
    sidebarTitle: 'Man Explorer',
    currentHash: '',
    pages: [],
    entries: [],
    searchQuery: '',
    searchResults: [],
    currentPage: null,
    contentState: 'empty',
    contentText: '',

    async init() {
        window.addEventListener('hashchange', () => this.handleHashChange());
        await this.loadPages();

        if (window.location.hash.length > 1) {
            this.handleHashChange();
        } else if (localStorage.getItem('lastHash')) {
            window.location.hash = localStorage.getItem('lastHash');
        }
    },

    updateHashState() {
        this.currentHash = window.location.hash.substring(1);
    },

    handleHashChange() {
        this.updateHashState();
        const hash = this.currentHash;
        const [page, name] = hash.split('/');
        const currentPage = this.pages.find((p) => p.page == page);
        if (!currentPage) return;

        if (this.currentPage != page) {
            PetiteVue.nextTick(() => (this.$refs.sidebarContent.scrollTop = 0));
        }
        this.currentPage = currentPage.page;
        this.entries = currentPage.names;

        localStorage.setItem('lastHash', window.location.hash);

        if (name !== undefined) {
            this.openManPage(currentPage.page, name);
        }
    },

    handleSearch() {
        const query = this.searchQuery.trim();
        if (query.length === 0) {
            this.searchResults = [];
            return;
        }

        const results = [];
        for (const page of this.pages) {
            for (const name of page.names) {
                if (name.toLowerCase().includes(query.toLowerCase())) {
                    results.push({ page: page.page, name });
                }
            }
        }
        results.sort((a, b) => a.name.localeCompare(b.name));
        this.searchResults = results;
        PetiteVue.nextTick(() => (this.$refs.sidebarContent.scrollTop = 0));
    },

    clearSearch() {
        this.searchQuery = '';
        this.searchResults = [];
    },

    async loadPages() {
        const res = await fetch('/api/man');
        this.pages = await res.json();
    },

    async openManPage(page, name) {
        document.title = `Man Explorer - ${page} - ${name}`;
        this.sidebarTitle = document.title;
        this.contentState = 'loading';
        const res = await fetch(`/api/man/${page}/${name}`);
        this.contentText = await res.text();
        this.contentState = 'loaded';
        PetiteVue.nextTick(() => (this.$refs.content.scrollTop = 0));
    },
}).mount('#app');
