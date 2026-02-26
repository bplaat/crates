/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

if (navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh')) {
    document.body.classList.add('is-bwebview-macos');
}

window.addEventListener('contextmenu', (e) => e.preventDefault());

const PAGE_SIZE = 100;

PetiteVue.createApp({
    sidebarTitle: 'Sequel Explorer',
    dbPath: localStorage.getItem('lastDbPath') || '',
    openBtnDisabled: false,
    dbOpened: false,
    tables: [],
    currentTable: null,
    activeTab: 'data',
    rowCount: '',
    columns: [],
    showDataTable: false,
    showDataLoading: false,
    showDataEmpty: false,
    dataEmptyText: 'No rows',
    schemaText: '',
    queryText: '',
    isCustomQuery: false,
    currentOffset: 0,
    currentTotal: 0,
    isLoading: false,

    init() {
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0].isIntersecting && this.currentTable && !this.isCustomQuery) {
                    this.loadMoreRows(this.currentTable);
                }
            },
            { rootMargin: '200px' },
        );
        observer.observe(this.$refs.loadSentinel);
    },

    async openDatabase() {
        const path = this.dbPath.trim();
        if (!path) return;
        this.openBtnDisabled = true;
        localStorage.setItem('lastDbPath', this.dbPath);
        const res = await fetch('/api/open', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path }),
        });
        const data = await res.json();
        this.openBtnDisabled = false;
        if (data.error) {
            alert('Failed to open database:\n' + data.error);
            return;
        }
        this.dbOpened = true;
        await this.loadTables();
    },

    async loadTables() {
        const res = await fetch('/api/tables');
        this.tables = await res.json();
    },

    async selectTable(name) {
        if (name === this.currentTable) return;
        this.currentTable = name;
        this.isCustomQuery = false;
        this.queryText = '';
        await this.openTableView(name);
    },

    async openTableView(name) {
        const title = `Sequel Explorer - ${name}`;
        document.title = title;
        this.sidebarTitle = title;

        this.currentOffset = 0;
        this.currentTotal = 0;
        this.isCustomQuery = false;
        this.columns = [];
        this.rowCount = '';
        this.showDataEmpty = false;
        this.showDataLoading = true;
        this.showDataTable = false;
        this.activeTab = 'data';
        this.$refs.dataTbody.innerHTML = '';

        await this.loadMoreRows(name);

        fetch(`/api/table/${encodeURIComponent(name)}/schema`)
            .then((r) => r.json())
            .then((data) => {
                this.schemaText = data.error ? 'Error: ' + data.error : data.sql || '';
            })
            .catch((err) => {
                this.schemaText = 'Error loading schema: ' + err.message;
            });
    },

    async loadMoreRows(tableName) {
        if (this.isLoading) return;
        if (this.currentOffset > 0 && this.currentOffset >= this.currentTotal) return;

        this.isLoading = true;
        this.showDataLoading = true;

        const url = `/api/table/${encodeURIComponent(tableName)}/data?offset=${this.currentOffset}&limit=${PAGE_SIZE}`;
        const res = await fetch(url);
        const data = await res.json();

        this.showDataLoading = false;
        this.isLoading = false;

        if (data.error) {
            this.dataEmptyText = 'Error: ' + data.error;
            this.showDataEmpty = true;
            return;
        }

        this.currentTotal = data.total;
        this.rowCount = `${data.total.toLocaleString()} rows`;

        if (this.currentOffset === 0) {
            this.columns = data.columns;
            this.showDataTable = true;
            if (data.rows.length === 0) {
                this.dataEmptyText = 'No rows';
                this.showDataEmpty = true;
                return;
            }
        }

        this.appendRows(data.rows);
        this.currentOffset += data.rows.length;
    },

    appendRows(rows) {
        const frag = document.createDocumentFragment();
        for (const row of rows) {
            const tr = document.createElement('tr');
            for (const val of row) {
                const td = document.createElement('td');
                if (val === null) {
                    td.textContent = 'NULL';
                    td.classList.add('is-null');
                } else {
                    td.textContent = String(val);
                }
                tr.appendChild(td);
            }
            frag.appendChild(tr);
        }
        this.$refs.dataTbody.appendChild(frag);
    },

    async runQuery() {
        const sql = this.queryText.trim();
        if (!sql) return;

        this.isCustomQuery = true;
        this.activeTab = 'data';
        this.currentOffset = 0;
        this.currentTotal = 0;
        this.columns = [];
        this.rowCount = '';
        this.showDataEmpty = false;
        this.showDataLoading = true;
        this.showDataTable = false;
        this.$refs.dataTbody.innerHTML = '';

        const res = await fetch('/api/query', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ sql }),
        });
        const data = await res.json();
        this.showDataLoading = false;

        if (data.error) {
            this.dataEmptyText = 'Error: ' + data.error;
            this.showDataEmpty = true;
            return;
        }

        this.rowCount = `${data.rows.length.toLocaleString()} rows`;
        this.columns = data.columns;
        this.showDataTable = true;

        if (data.rows.length === 0) {
            this.dataEmptyText = 'No rows';
            this.showDataEmpty = true;
            return;
        }

        this.appendRows(data.rows);
    },

    clearQuery() {
        this.queryText = '';
        this.isCustomQuery = false;
        if (this.currentTable) this.openTableView(this.currentTable);
    },
}).mount('#app');
