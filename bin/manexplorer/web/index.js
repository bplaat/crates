/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

if (navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh')) {
    document.body.classList.add('is-bwebview-macos');
}

const sidebarTitle = document.getElementById('sidebar-title');
const searchButton = document.getElementById('search-button');
const searchInput = document.getElementById('search-input');
const searchClearButton = document.getElementById('search-clear-button');
const pagesElement = document.getElementById('pages');
const entriesElement = document.getElementById('entries');
const entriesEmptyElement = document.getElementById('entries-empty');
const searchResultsElement = document.getElementById('search-results');
const searchResultsEmptyElement = document.getElementById('search-results-empty');
const contentElement = document.getElementById('content');
const contentEmptyElement = document.getElementById('content-empty');
const contentLoadingElement = document.getElementById('content-loading');

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
let pages = [];

function searchPages(query) {
    const results = [];
    for (const page of pages) {
        for (const name of page.names) {
            if (name.toLowerCase().includes(query.toLowerCase())) {
                results.push({ page: page.page, name });
            }
        }
    }
    results.sort((a, b) => a.name.localeCompare(b.name));
    return results;
}

function updateLinksActive() {
    const links = document.querySelectorAll('a');
    for (const link of links) {
        link.classList.remove('is-active');
        if (
            (link.href.split('#')[1].split('/').length == 1 && window.location.href.startsWith(link.href)) ||
            link.href == window.location.href
        ) {
            link.classList.add('is-active');
        }
    }
}

function showSidebarContainer(el) {
    pagesElement.classList.add('hidden');
    entriesElement.classList.add('hidden');
    entriesEmptyElement.classList.add('hidden');
    searchResultsElement.classList.add('hidden');
    searchResultsEmptyElement.classList.add('hidden');
    if (el == pagesElement) {
        pagesElement.classList.remove('hidden');
        entriesElement.classList.remove('hidden');
    }
    if (el == searchResultsElement) searchResultsElement.classList.remove('hidden');
    if (el == searchResultsEmptyElement) searchResultsEmptyElement.classList.remove('hidden');
}

function showContentContainer(el) {
    contentElement.classList.add('hidden');
    contentEmptyElement.classList.add('hidden');
    contentLoadingElement.classList.add('hidden');
    if (el == contentElement) contentElement.classList.remove('hidden');
    if (el == contentEmptyElement) contentEmptyElement.classList.remove('hidden');
    if (el == contentLoadingElement) contentLoadingElement.classList.remove('hidden');
}

async function loadPages() {
    const res = await fetch('/man');
    pages = await res.json();
    pagesElement.innerHTML = `<ul>${pages
        .map((page) => `<li><a href="#${page.page}">${page.page}. ${pageNames[page.page]}</a></li>`)
        .join('')}</ul>`;

    if (localStorage.getItem('lastPage')) {
        const lastPage = localStorage.getItem('lastPage');
        const lastName = localStorage.getItem('lastName');
        if (lastName != null) {
            window.location.hash = `#${lastPage}/${lastName}`;
        } else {
            window.location.hash = `#${lastPage}`;
        }
    }
}

async function loadPage(page, name) {
    document.title = `Man Explorer - ${page} - ${name}`;
    sidebarTitle.textContent = document.title;
    showContentContainer(contentLoadingElement);

    const res = await fetch(`/man/${page}/${name}`);
    const text = await res.text();

    showContentContainer(contentElement);
    contentElement.textContent = text;
    contentElement.scrollTop = 0;
}

function handleSearchChange(value) {
    const query = value.trim();
    if (query.length == 0) {
        searchClearButton.disabled = true;
        showSidebarContainer(pagesElement);
        return;
    }
    searchClearButton.disabled = false;

    const results = searchPages(query);
    if (results.length == 0) {
        showSidebarContainer(searchResultsEmptyElement);
        return;
    }

    searchResultsElement.innerHTML = `<ul>${results
        .map((result) => `<li><a href="#${result.page}/${result.name}">${result.name}</a></li>`)
        .join('')}</ul>`;

    showSidebarContainer(searchResultsElement);
    updateLinksActive();
}

searchButton.addEventListener('click', () => searchInput.focus());
searchInput.addEventListener('input', (event) => handleSearchChange(event.target.value));
searchClearButton.addEventListener('click', () => {
    searchInput.value = '';
    handleSearchChange('');
});

window.addEventListener('contextmenu', (event) => event.preventDefault());
window.addEventListener('hashchange', () => {
    const hash = window.location.hash.substring(1);
    const [page, name] = hash.split('/');
    const currentPage = pages.find((entry) => entry.page == page);

    entriesElement.innerHTML = `<ul>${currentPage.names
        .map((name) => `<li><a href="#${currentPage.page}/${name}">${name}</a></li>`)
        .join('')}</ul>`;

    if (searchInput.value.trim().length == 0) {
        showSidebarContainer(pagesElement);
    }

    localStorage.setItem('lastPage', page);
    localStorage.setItem('lastName', name || null);
    updateLinksActive();

    if (name != undefined) {
        loadPage(currentPage.page, name);
    }
});
loadPages();
