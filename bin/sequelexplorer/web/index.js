/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

const isMacosBwebview = navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh');
if (isMacosBwebview) {
    document.body.classList.add('is-bwebview-macos');
}

window.addEventListener('contextmenu', (e) => e.preventDefault());

const PAGE_SIZE = 100;

// Every action the user can trigger, keyed by the action id used in the macOS menu bar.
// `method` is the app method that performs it, `key`/`shift`/`alt` are the Command-chord
// fallback for platforms without a menu bar. Keep the ids and chords in sync with main.rs.
const ACTIONS = {
    open: { method: 'openDatabase', key: 'o' },
    showData: { method: 'showData', key: '1' },
    showSchema: { method: 'showSchema', key: '2' },
    runQuery: { method: 'runQuery', key: 'Enter' },
    clearQuery: { method: 'clearQuery', key: 'k' },
};

// Returns the id of the action this key event triggers, or null when it is not a shortcut
function matchShortcut(event) {
    if (!event.metaKey && !event.ctrlKey) return null;
    const pressed = event.key.toLowerCase();
    const match = Object.entries(ACTIONS).find(
        ([, { key, shift = false, alt = false }]) =>
            key.toLowerCase() === pressed && event.shiftKey === shift && event.altKey === alt,
    );
    return match ? match[0] : null;
}

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
    isQueryRunning: false,
    currentOffset: 0,
    currentTotal: 0,
    isLoading: false,

    async init() {
        window.ipc.addEventListener('message', (event) => {
            const message = JSON.parse(event.data);
            if (message.type === 'openFile') this._openDatabaseByPath(message.path);
            if (message.type === 'restoreLastFile') {
                const lastDbPath = localStorage.getItem('lastDbPath');
                if (lastDbPath) this._openDatabaseByPath(lastDbPath);
            }
            if (message.type === 'menuAction') this.performAction(message.action);
        });
        // On macOS the native menu bar already claims these chords and delivers them over IPC,
        // so only the other platforms need to resolve them here
        if (!isMacosBwebview) {
            window.addEventListener('keydown', (event) => {
                const action = matchShortcut(event);
                if (!action) return;
                event.preventDefault();
                this.performAction(action);
            });
        }
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0].isIntersecting && this.currentTable && !this.isCustomQuery) {
                    this.loadMoreRows(this.currentTable);
                }
            },
            { rootMargin: '200px' },
        );
        observer.observe(this.$refs.loadSentinel);

        ipcSend('ready');
    },

    // Runs an action by id, coming from the macOS menu bar or from a keyboard shortcut
    performAction(id) {
        const action = ACTIONS[id];
        if (action) this[action.method]();
    },

    showData() {
        if (this.currentTable !== null) this.activeTab = 'data';
    },

    showSchema() {
        if (this.currentTable !== null) this.activeTab = 'schema';
    },

    async openDatabase() {
        const { path } = await ipcRequest('openFileDialog');
        if (!path) return;
        await this._openDatabaseByPath(path);
    },

    async _openDatabaseByPath(path) {
        const { ok, error } = await ipcRequest('openDatabase', { path });
        if (!ok) {
            if (localStorage.getItem('lastDbPath') === path) {
                localStorage.removeItem('lastDbPath');
            }
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
            const bytes = Uint8Array.fromBase64(val);
            const hex = Array.from(bytes)
                .map((b) => b.toString(16).padStart(2, '0'))
                .join('');
            return `X'${hex.toUpperCase()}'`;
        } catch (e) {
            return `'${String(val).replace(/'/g, "''")}'`;
        }
    },

    async navigateToForeignKey(table, column, value) {
        this.queryText = `SELECT * FROM "${table}" WHERE "${column}" = ${this.formatSqlValue(value)}`;
        await this.runQuery();
    },

    async runQuery() {
        const sql = this.queryText.trim();
        // Guard against re-entry, the same query can be triggered by the input, the button and a shortcut
        if (!sql || this.isQueryRunning) return;

        this.isQueryRunning = true;
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

        let data;
        try {
            const res = await fetch('/api/query', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ sql }),
            });
            data = await res.json();
        } finally {
            this.isQueryRunning = false;
            this.showDataLoading = false;
        }

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
                const bytes = Uint8Array.fromBase64(val);
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
