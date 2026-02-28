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

function ipcSend(type, data = {}) {
    window.ipc.postMessage(JSON.stringify({ type, ...data }));
}

async function ipcRequest(type, data = {}) {
    return new Promise((resolve) => {
        const listener = (event) => {
            const message = JSON.parse(event.data);
            if (message.type === `${type}Response`) {
                window.ipc.removeEventListener('message', listener);
                resolve(message);
            }
        };
        window.ipc.addEventListener('message', listener);
        ipcSend(type, data);
    });
}

PetiteVue.createApp({
    dbPath: '',
    dbFileName: '',
    dbOpened: false,
    tables: [],
    currentTable: null,
    activeTab: 'data',
    rowCount: '',
    columns: [],
    rows: [],
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

    async init() {
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0].isIntersecting && this.currentTable && !this.isCustomQuery) {
                    this.loadMoreRows(this.currentTable);
                }
            },
            { rootMargin: '200px' },
        );
        observer.observe(this.$refs.loadSentinel);

        const lastDbPath = localStorage.getItem('lastDbPath');
        if (lastDbPath) {
            await this._openDatabaseByPath(lastDbPath);
        }
    },

    async openDatabase() {
        const { path } = await ipcRequest('openFileDialog');
        if (!path) return;
        await this._openDatabaseByPath(path);
    },

    async _openDatabaseByPath(path) {
        const { ok, error } = await ipcRequest('openDatabase', { path });
        if (!ok) {
            alert('Failed to open database:\n' + error);
            return;
        }
        this.dbPath = path;
        this.dbFileName = path.replace(/.*[\\/]/, '');
        localStorage.setItem('lastDbPath', path);
        this.dbOpened = true;
        document.title = `Sequel Explorer - ${this.dbFileName}`;
        await this.loadTables();
        const lastTable = localStorage.getItem('lastTableName');
        if (lastTable && this.tables.includes(lastTable)) {
            await this.selectTable(lastTable);
        }
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
        localStorage.setItem('lastTableName', name);
        await this.openTableView(name);
    },

    async openTableView(name) {
        document.title = `Sequel Explorer - ${this.dbFileName} - ${name}`;

        this.currentOffset = 0;
        this.currentTotal = 0;
        this.isCustomQuery = false;
        this.columns = [];
        this.rows = [];
        this.rowCount = '';
        this.showDataEmpty = false;
        this.showDataLoading = true;
        this.showDataTable = false;
        this.activeTab = 'data';

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
        this.rows = this.rows.concat(rows);
    },

    formatSqlValue(val) {
        if (val === null) {
            return 'NULL';
        }

        if (typeof val === 'number') {
            return String(val);
        }

        try {
            const binaryStr = atob(val);
            const bytes = new Uint8Array(binaryStr.length);
            for (let i = 0; i < binaryStr.length; i++) {
                bytes[i] = binaryStr.charCodeAt(i);
            }
            const hex = Array.from(bytes)
                .map((b) => b.toString(16).padStart(2, '0'))
                .join('');
            return `X'${hex.toUpperCase()}'`;
        } catch (e) {
            return `'${String(val).replace(/'/g, "''")}'`;
        }
    },

    async navigateToForeignKey(table, column, value) {
        const sql = `SELECT * FROM "${table}" WHERE "${column}" = ${this.formatSqlValue(value)}`;
        this.queryText = sql;
        this.isCustomQuery = true;
        this.activeTab = 'data';
        this.currentOffset = 0;
        this.currentTotal = 0;
        this.columns = [];
        this.rows = [];
        this.rowCount = '';
        this.showDataEmpty = false;
        this.showDataLoading = true;
        this.showDataTable = false;

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

    async runQuery() {
        const sql = this.queryText.trim();
        if (!sql) return;

        this.isCustomQuery = true;
        this.activeTab = 'data';
        this.currentOffset = 0;
        this.currentTotal = 0;
        this.columns = [];
        this.rows = [];
        this.rowCount = '';
        this.showDataEmpty = false;
        this.showDataLoading = true;
        this.showDataTable = false;

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

    formatCellValue(val, colIdx) {
        if (val === null) return 'NULL';

        const column = this.columns[colIdx];
        if (column.is_blob) {
            try {
                const binaryStr = atob(val);
                const bytes = new Uint8Array(binaryStr.length);
                for (let i = 0; i < binaryStr.length; i++) {
                    bytes[i] = binaryStr.charCodeAt(i);
                }

                if (bytes.length === 16) {
                    const hex = Array.from(bytes)
                        .map((b) => b.toString(16).padStart(2, '0'))
                        .join('');
                    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
                }

                return Array.from(bytes)
                    .map((b) => b.toString(16).padStart(2, '0'))
                    .join('');
            } catch (e) {
                return val;
            }
        }

        return val;
    },
}).mount('#app');
