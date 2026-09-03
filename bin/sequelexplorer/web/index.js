/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

const isMacosBwebview = navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh');
if (isMacosBwebview) {
    document.body.classList.add('is-bwebview-macos');
}

const PAGE_SIZE = 100;
const supportsUnixSocket = !navigator.userAgent.includes('Windows');

// Every action the user can trigger, keyed by the action id used in the macOS menu bar.
// `method` is the app method that performs it, `key`/`shift`/`alt` are the Command-chord
// fallback for platforms without a menu bar. Keep the ids and chords in sync with main.rs.
const ACTIONS = {
    open: { method: 'openDatabase', key: 'o' },
    importSql: { method: 'importSql' },
    exportSql: { method: 'exportSql' },
    showData: { method: 'showData', key: '1' },
    showSchema: { method: 'showSchema', key: '2' },
    showQuery: { method: 'showQuery', key: '3' },
    runQuery: { method: 'runQuery', key: 'Enter' },
    clearQuery: { method: 'clearQuery', key: 'k' },
};

// Returns the id of the action this key event triggers, or null when it is not a shortcut
function matchShortcut(event) {
    if (!event.metaKey && !event.ctrlKey) return null;
    const pressed = event.key.toLowerCase();
    const match = Object.entries(ACTIONS).find(
        ([, { key, shift = false, alt = false }]) =>
            key?.toLowerCase() === pressed && event.shiftKey === shift && event.altKey === alt,
    );
    return match ? match[0] : null;
}

function ipcSend(type, data = {}) {
    window.ipc.postMessage(JSON.stringify({ type, ...data }));
}

let nextRequestId = 1;
function ipcRequest(type, data = {}) {
    const requestId = nextRequestId++;
    return new Promise((resolve) => {
        const listener = (event) => {
            const message = JSON.parse(event.data);
            if (message.type === `${type}Response` && message.requestId === requestId) {
                window.ipc.removeEventListener('message', listener);
                resolve(message);
            }
        };
        window.ipc.addEventListener('message', listener);
        window.ipc.postMessage(JSON.stringify({ type, requestId, ...data }));
    });
}

function formatSqlValue(cell) {
    if (cell.kind === 'null') return 'NULL';
    if (cell.kind === 'integer' || cell.kind === 'float') return String(cell.value);
    if (cell.kind === 'blob') {
        const bytes = Uint8Array.fromBase64(cell.value);
        const hex = Array.from(bytes)
            .map((byte) => byte.toString(16).padStart(2, '0'))
            .join('');
        return `X'${hex.toUpperCase()}'`;
    }
    return `'${String(cell.value).replace(/'/g, "''")}'`;
}

function formatCellValue(cell, isBlob) {
    if (cell.kind === 'null') return 'NULL';
    if (!isBlob && cell.kind !== 'blob') return cell.value;

    try {
        const bytes = Uint8Array.fromBase64(cell.value);
        if (bytes.length === 16) {
            const hex = Array.from(bytes)
                .map((byte) => byte.toString(16).padStart(2, '0'))
                .join('');
            return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
        }
        return Array.from(bytes)
            .map((byte) => byte.toString(16).padStart(2, '0'))
            .join('');
    } catch (_error) {
        return cell.value;
    }
}

PetiteVue.createApp({
    dbPath: '',
    dbFileName: '',
    dbOpened: false,
    connectionKind: '',
    connectionLabel: '',
    connectionTab: 'sqlite',
    connectionError: '',
    browserError: '',
    isConnecting: false,
    isSelectingDatabase: false,
    databases: [],
    currentDatabase: '',
    activeDatabase: '',
    mysqlUsers: [],
    assignableDatabases: [],
    isLoadingUsers: false,
    isSavingUser: false,
    usersError: '',
    editingUser: false,
    isCreatingUser: false,
    selectedUserKey: '',
    userForm: {
        user: '',
        host: '%',
        password: '',
        databases: [],
        oldUser: null,
        oldHost: null,
    },
    supportsUnixSocket,
    mysql: {
        transport: 'tcp',
        host: '127.0.0.1',
        port: 3306,
        socket: '/tmp/mysql.sock',
        user: 'root',
        password: '',
        tls: false,
        remember: true,
    },
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
    schemaSql: '',
    schemaTableName: '',
    schemaColumns: [],
    schemaIndexes: [],
    schemaError: '',
    schemaLoading: false,
    schemaSaving: false,
    newSchemaColumn: {
        name: '',
        type: 'TEXT',
        nullable: true,
        defaultSql: '',
        primaryKey: false,
        primaryKeyPosition: 0,
        autoIncrement: false,
        generated: false,
        characterSet: null,
        collation: null,
        comment: '',
        extra: '',
    },
    newSchemaIndex: { name: '', columns: [], unique: false, primary: false, readOnly: false },
    queryText: '',
    isCustomQuery: false,
    isQueryRunning: false,
    currentOffset: 0,
    currentTotal: 0,
    currentCursor: null,
    isLoading: false,
    viewGeneration: 0,
    editingCell: null,
    editValue: '',
    cellEditError: '',
    rawQueryText: '',
    rawQueryColumns: [],
    rawQueryRows: [],
    rawQueryStatus: 'Ready',
    rawQueryError: '',
    rawQueryRunning: false,
    rawQueryHasRun: false,
    newRowFields: [],
    addRowError: '',
    isAddingRow: false,
    rowMenu: null,
    isDeletingRow: false,
    sqlTransferRunning: false,

    async init() {
        window.addEventListener('click', () => this.closeRowMenu());
        window.addEventListener('blur', () => this.closeRowMenu());
        window.addEventListener('resize', () => this.closeRowMenu());
        window.ipc.addEventListener('message', (event) => {
            const message = JSON.parse(event.data);
            if (message.type === 'openFile') this._openDatabaseByPath(message.path);
            if (message.type === 'restoreLastFile') {
                const lastDbPath = localStorage.getItem('lastDbPath');
                if (lastDbPath) {
                    this._openDatabaseByPath(lastDbPath);
                } else {
                    this.restoreMysqlConnection();
                }
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

        const notifyReady = () => requestAnimationFrame(() => requestAnimationFrame(() => ipcSend('ready')));
        if (document.readyState === 'complete') {
            notifyReady();
        } else {
            window.addEventListener('load', notifyReady, { once: true });
        }
    },

    // Runs an action by id, coming from the macOS menu bar or from a keyboard shortcut
    performAction(id) {
        const action = ACTIONS[id];
        if (action) this[action.method]();
    },

    showData() {
        this.activeTab = 'data';
    },

    showSchema() {
        if (this.currentTable !== null) this.activeTab = 'schema';
    },

    schemaColumnPayload(column) {
        return {
            ...column,
            name: column.name.trim(),
            type: column.type.trim(),
            defaultSql: column.defaultSql?.trim() || null,
        };
    },

    schemaColumnChanged(column) {
        return (
            column.name.trim() !== column.originalName ||
            column.type.trim() !== column.originalType ||
            column.nullable !== column.originalNullable ||
            (column.defaultSql?.trim() || '') !== column.originalDefaultSql ||
            column.primaryKey !== column.originalPrimaryKey ||
            column.autoIncrement !== column.originalAutoIncrement
        );
    },

    schemaIndexChanged(index) {
        return (
            index.name.trim() !== index.originalName ||
            index.unique !== index.originalUnique ||
            index.columns.join('\0') !== index.originalColumns.join('\0')
        );
    },

    async applySchemaChange(change, confirmation = '') {
        if (!this.currentTable || this.schemaSaving) return false;
        if (confirmation && !confirm(confirmation)) return false;
        this.schemaSaving = true;
        this.schemaError = '';
        const table = this.currentTable;
        try {
            const response = await fetch(`/api/table/${encodeURIComponent(table)}/schema`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(change),
            });
            const data = await response.json();
            if (data.error) {
                this.schemaError = data.error;
                return false;
            }
            this.currentTable = data.table;
            localStorage.setItem('lastTableName', data.table);
            await this.loadTables();
            await this.openTableView(data.table);
            this.activeTab = 'schema';
            return true;
        } catch (error) {
            this.schemaError = `Schema change failed: ${error.message}`;
            return false;
        } finally {
            this.schemaSaving = false;
        }
    },

    async renameSchemaTable() {
        const newName = this.schemaTableName.trim();
        if (!newName || newName === this.currentTable) return;
        await this.applySchemaChange(
            { action: 'renameTable', newName },
            `Rename table ${this.currentTable} to ${newName}?`,
        );
    },

    async saveSchemaColumn(column) {
        if (!this.schemaColumnChanged(column)) return;
        const definitionChanged =
            column.type.trim() !== column.originalType ||
            column.nullable !== column.originalNullable ||
            (column.defaultSql?.trim() || '') !== column.originalDefaultSql ||
            column.primaryKey !== column.originalPrimaryKey ||
            column.autoIncrement !== column.originalAutoIncrement;
        await this.applySchemaChange(
            {
                action: 'updateColumn',
                oldName: column.originalName,
                column: this.schemaColumnPayload(column),
            },
            definitionChanged
                ? `Change the definition of ${column.originalName}? The database may rewrite or validate existing rows.`
                : '',
        );
    },

    async addSchemaColumn() {
        if (
            await this.applySchemaChange({
                action: 'addColumn',
                column: this.schemaColumnPayload(this.newSchemaColumn),
            })
        ) {
            this.newSchemaColumn = {
                ...this.newSchemaColumn,
                name: '',
                type: 'TEXT',
                nullable: true,
                defaultSql: '',
                autoIncrement: false,
            };
        }
    },

    async dropSchemaColumn(column) {
        await this.applySchemaChange(
            { action: 'dropColumn', name: column.originalName },
            `Delete column ${column.originalName} and all of its data? This cannot be undone.`,
        );
    },

    async addSchemaIndex() {
        if (
            await this.applySchemaChange({
                action: 'addIndex',
                index: { ...this.newSchemaIndex, name: this.newSchemaIndex.name.trim() },
            })
        ) {
            this.newSchemaIndex = { name: '', columns: [], unique: false, primary: false, readOnly: false };
        }
    },

    async saveSchemaIndex(index) {
        if (!this.schemaIndexChanged(index)) return;
        await this.applySchemaChange({
            action: 'updateIndex',
            oldName: index.originalName,
            index: { ...index, name: index.name.trim() },
        });
    },

    async dropSchemaIndex(index) {
        await this.applySchemaChange(
            { action: 'dropIndex', name: index.originalName },
            `Delete index ${index.originalName}?`,
        );
    },

    showQuery() {
        this.activeTab = 'query';
    },

    async importSql() {
        if (!this.dbOpened || this.sqlTransferRunning) return;
        if (this.connectionKind === 'mysql' && !this.currentDatabase) {
            alert('Select a MySQL database before importing SQL.');
            return;
        }
        if (!confirm('Import a SQL file into the current database? The script may modify or delete data.')) return;
        this.sqlTransferRunning = true;
        try {
            const { cancelled, error } = await ipcRequest('importSql');
            if (cancelled) return;
            if (error) {
                this.resetBrowser();
                await this.loadTables();
                alert(`SQL import failed:\n${error}`);
                return;
            }
            this.resetBrowser();
            await this.loadTables();
            alert('SQL import completed.');
        } finally {
            this.sqlTransferRunning = false;
        }
    },

    async exportSql() {
        if (!this.dbOpened || this.sqlTransferRunning) return;
        if (this.connectionKind === 'mysql' && !this.currentDatabase) {
            alert('Select a MySQL database before exporting SQL.');
            return;
        }
        this.sqlTransferRunning = true;
        try {
            const baseName = (this.currentDatabase || this.dbFileName || 'database')
                .replace(/\.(db|sqlite|sqlite3)$/i, '')
                .replace(/[^a-z0-9._-]+/gi, '_');
            const { cancelled, error } = await ipcRequest('exportSql', { fileName: `${baseName || 'database'}.sql` });
            if (cancelled) return;
            if (error) {
                alert(`SQL export failed:\n${error}`);
                return;
            }
            alert('SQL export completed.');
        } finally {
            this.sqlTransferRunning = false;
        }
    },

    openAddRowDialog() {
        if (!this.currentTable || this.isCustomQuery || this.columns.length === 0) return;
        this.addRowError = '';
        this.newRowFields = this.columns.map((column) => {
            const type = column.type.toUpperCase();
            return {
                name: column.name,
                type: column.type,
                isBlob: column.is_blob,
                isPrimaryKey: column.is_primary_key,
                inputMode: /INT/.test(type) ? 'numeric' : /REAL|FLOAT|DOUBLE/.test(type) ? 'decimal' : 'text',
                mode: 'value',
                value: '',
            };
        });
        this.$refs.addRowDialog.showModal();
        requestAnimationFrame(() => this.$refs.addRowDialog.querySelector('input')?.focus());
    },

    closeAddRowDialog() {
        if (this.isAddingRow) return;
        if (this.$refs.addRowDialog.open) this.$refs.addRowDialog.close();
        this.newRowFields = [];
        this.addRowError = '';
    },

    openRowMenu(rowIdx, event) {
        if (this.isCustomQuery || !this.currentTable || this.editingCell) return;
        const keyEntries = this.columns
            .map((column, index) => [column, this.rows[rowIdx][index]])
            .filter(([column]) => column.is_primary_key);
        if (keyEntries.length === 0) return;

        event.preventDefault();
        this.rowMenu = {
            table: this.currentTable,
            generation: this.viewGeneration,
            keys: keyEntries.map(([column, value]) => ({ name: column.name, value })),
            x: Math.max(8, Math.min(event.clientX, window.innerWidth - 184)),
            y: Math.max(8, Math.min(event.clientY, window.innerHeight - 56)),
        };
    },

    closeRowMenu() {
        if (!this.isDeletingRow) this.rowMenu = null;
    },

    async deleteMenuRow() {
        const menu = this.rowMenu;
        if (!menu || this.isDeletingRow) return;
        if (!window.confirm(`Delete this row from ${menu.table}? This cannot be undone.`)) {
            this.rowMenu = null;
            return;
        }

        const keys = Object.fromEntries(menu.keys.map(({ name, value }) => [name, value]));
        this.isDeletingRow = true;
        this.cellEditError = '';
        try {
            const response = await fetch(`/api/table/${encodeURIComponent(menu.table)}/data`, {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ keys }),
            });
            const data = await response.json();
            if (menu.generation !== this.viewGeneration || menu.table !== this.currentTable) return;
            if (data.error) throw new Error(data.error);
            this.rowMenu = null;
            await this.openTableView(menu.table);
        } catch (error) {
            if (menu.generation !== this.viewGeneration || menu.table !== this.currentTable) return;
            this.cellEditError = `Could not delete row: ${error.message}`;
        } finally {
            this.isDeletingRow = false;
            if (menu.generation !== this.viewGeneration || menu.table !== this.currentTable) {
                this.rowMenu = null;
            }
        }
    },

    newRowValue(field) {
        if (field.mode === 'default') return null;
        if (field.mode === 'null') return { kind: 'null', value: null };
        const type = field.type.toUpperCase();
        if (field.isBlob) return { kind: 'blob', value: field.value.trim() };
        if (/INT/.test(type)) {
            if (!/^[-+]?\d+$/.test(field.value.trim())) throw new Error(`${field.name} must be an integer`);
            return { kind: 'integer', value: field.value.trim() };
        }
        if (/REAL|FLOAT|DOUBLE/.test(type)) {
            const value = Number(field.value);
            if (field.value.trim() === '' || !Number.isFinite(value)) {
                throw new Error(`${field.name} must be a number`);
            }
            return { kind: 'float', value };
        }
        return { kind: 'text', value: field.value };
    },

    async insertRow() {
        if (this.isAddingRow || !this.currentTable) return;
        this.addRowError = '';
        let values;
        try {
            values = Object.fromEntries(this.newRowFields.map((field) => [field.name, this.newRowValue(field)]));
        } catch (error) {
            this.addRowError = error.message;
            return;
        }

        this.isAddingRow = true;
        const table = this.currentTable;
        const generation = this.viewGeneration;
        try {
            const response = await fetch(`/api/table/${encodeURIComponent(table)}/data`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ values }),
            });
            const data = await response.json();
            if (generation !== this.viewGeneration || table !== this.currentTable) return;
            if (data.error) {
                this.addRowError = data.error;
                return;
            }
            this.isAddingRow = false;
            this.closeAddRowDialog();
            await this.openTableView(table);
        } catch (error) {
            if (generation !== this.viewGeneration || table !== this.currentTable) return;
            this.addRowError = `Could not add row: ${error.message}`;
        } finally {
            if (generation === this.viewGeneration && table === this.currentTable) {
                this.isAddingRow = false;
            }
        }
    },

    async openDatabase() {
        if (this.sqlTransferRunning || this.schemaSaving) return;
        this.connectionError = '';
        this.$refs.connectionDialog.showModal();
    },

    closeConnectionDialog() {
        if (this.$refs.connectionDialog.open) this.$refs.connectionDialog.close();
    },

    async openUsersDialog() {
        if (this.connectionKind !== 'mysql') return;
        this.usersError = '';
        this.cancelEditingUser();
        this.$refs.usersDialog.showModal();
        await this.loadUsers();
    },

    closeUsersDialog() {
        if (this.$refs.usersDialog.open) this.$refs.usersDialog.close();
        this.cancelEditingUser();
    },

    async loadUsers() {
        this.isLoadingUsers = true;
        this.usersError = '';
        try {
            const response = await fetch('/api/users');
            const data = await response.json();
            if (data.error) {
                this.mysqlUsers = [];
                this.usersError = data.error;
                return;
            }
            this.mysqlUsers = data;
        } catch (error) {
            this.mysqlUsers = [];
            this.usersError = `Failed to load users: ${error.message}`;
        } finally {
            this.isLoadingUsers = false;
        }
    },

    availableUserDatabases(extra = []) {
        const systemDatabases = new Set(['information_schema', 'mysql', 'performance_schema', 'sys']);
        return [...new Set([...this.databases.filter((database) => !systemDatabases.has(database)), ...extra])].sort();
    },

    startCreatingUser() {
        this.usersError = '';
        this.editingUser = true;
        this.isCreatingUser = true;
        this.selectedUserKey = '';
        this.assignableDatabases = this.availableUserDatabases();
        this.userForm = {
            user: '',
            host: '%',
            password: '',
            databases:
                this.currentDatabase && this.assignableDatabases.includes(this.currentDatabase)
                    ? [this.currentDatabase]
                    : [],
            oldUser: null,
            oldHost: null,
        };
    },

    startEditingUser(account) {
        this.usersError = '';
        this.editingUser = true;
        this.isCreatingUser = false;
        this.selectedUserKey = `${account.user}@${account.host}`;
        this.assignableDatabases = this.availableUserDatabases(account.databases);
        this.userForm = {
            user: account.user,
            host: account.host,
            password: '',
            databases: [...account.databases],
            oldUser: account.user,
            oldHost: account.host,
        };
    },

    cancelEditingUser() {
        this.editingUser = false;
        this.isCreatingUser = false;
        this.selectedUserKey = '';
        this.usersError = '';
    },

    async saveUser() {
        if (this.isSavingUser) return;
        this.isSavingUser = true;
        this.usersError = '';
        try {
            const response = await fetch('/api/users', {
                method: this.isCreatingUser ? 'POST' : 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(this.userForm),
            });
            const data = await response.json();
            if (data.error) {
                const saveError = data.error;
                if (data.reload) {
                    const oldKey = `${this.userForm.oldUser}@${this.userForm.oldHost}`;
                    const newKey = `${this.userForm.user}@${this.userForm.host}`;
                    await this.loadUsers();
                    const currentUser = this.mysqlUsers.find((account) => {
                        const key = `${account.user}@${account.host}`;
                        return key === oldKey || key === newKey;
                    });
                    if (currentUser) this.startEditingUser(currentUser);
                }
                this.usersError = saveError;
                return;
            }
            const savedKey = `${this.userForm.user}@${this.userForm.host}`;
            await this.loadUsers();
            const savedUser = this.mysqlUsers.find((account) => `${account.user}@${account.host}` === savedKey);
            if (savedUser) this.startEditingUser(savedUser);
        } catch (error) {
            this.usersError = `Failed to save user: ${error.message}`;
        } finally {
            this.isSavingUser = false;
        }
    },

    async deleteUser() {
        if (this.isCreatingUser || this.isSavingUser) return;
        const account = `${this.userForm.oldUser}@${this.userForm.oldHost}`;
        if (!confirm(`Delete ${account}?\n\nThis immediately removes the account and all of its grants.`)) return;
        this.isSavingUser = true;
        this.usersError = '';
        try {
            const response = await fetch('/api/users', {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ user: this.userForm.oldUser, host: this.userForm.oldHost }),
            });
            const data = await response.json();
            if (data.error) {
                this.usersError = data.error;
                return;
            }
            this.cancelEditingUser();
            await this.loadUsers();
        } catch (error) {
            this.usersError = `Failed to delete user: ${error.message}`;
        } finally {
            this.isSavingUser = false;
        }
    },

    userInitial(user) {
        return (user || '?').slice(0, 1).toUpperCase();
    },

    databaseAccessLabel(account) {
        if (account.databases.length === 0) return 'No full direct grants';
        if (account.databases.length === 1) return `Full access to ${account.databases[0]}`;
        return `${account.databases.length} full direct grants`;
    },

    async openSqliteDatabase() {
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
        this.connectionKind = 'sqlite';
        this.connectionLabel = this.dbFileName;
        this.currentDatabase = this.dbFileName;
        this.activeDatabase = '';
        this.databases = [];
        localStorage.setItem('lastDbPath', path);
        this.dbOpened = true;
        this.closeConnectionDialog();
        this.resetBrowser();
        document.title = `Sequel Explorer - ${this.dbFileName}`;
        await this.loadTables();
        const lastTable = localStorage.getItem('lastTableName');
        if (lastTable && this.tables.includes(lastTable)) {
            await this.selectTable(lastTable);
        }
    },

    async restoreMysqlConnection() {
        const savedConnection = localStorage.getItem('lastMysqlConnection');
        if (!savedConnection) return;
        try {
            const connection = JSON.parse(savedConnection);
            if (!connection || !connection.transport || !connection.user) return;
            this.mysql = { ...this.mysql, ...connection, password: '', remember: true };
            this.connectionTab = 'mysql';
            if (!(await this._openMysql(true))) this.$refs.connectionDialog.showModal();
        } catch (_error) {
            localStorage.removeItem('lastMysqlConnection');
        }
    },

    async openMysql() {
        return this._openMysql(false);
    },

    async _openMysql(useSavedPassword) {
        if (this.isConnecting) return;
        this.isConnecting = true;
        this.connectionError = '';
        let response;
        try {
            let previousConnection = null;
            try {
                const savedConnection = JSON.parse(localStorage.getItem('lastMysqlConnection'));
                if (savedConnection?.transport && savedConnection?.user) previousConnection = savedConnection;
            } catch (_error) {
                localStorage.removeItem('lastMysqlConnection');
            }
            response = await ipcRequest('openMysql', {
                ...this.mysql,
                password: useSavedPassword ? null : this.mysql.password,
                previousConnection,
            });
        } finally {
            this.isConnecting = false;
        }
        if (!response.ok) {
            this.connectionError = response.error || 'Failed to connect';
            return false;
        }

        this.mysql.password = '';
        if (this.mysql.remember && response.credentialSaved) {
            const { transport, host, port, socket, user, tls } = this.mysql;
            localStorage.setItem('lastMysqlConnection', JSON.stringify({ transport, host, port, socket, user, tls }));
        } else {
            localStorage.removeItem('lastMysqlConnection');
        }
        if (response.credentialError) {
            alert(`Connected, but credentials could not be saved securely:\n${response.credentialError}`);
        }

        localStorage.removeItem('lastDbPath');
        this.dbPath = '';
        this.dbFileName = this.mysql.transport === 'tcp' ? this.mysql.host : this.mysql.socket;
        this.connectionKind = 'mysql';
        this.connectionLabel =
            this.mysql.transport === 'tcp'
                ? `${this.mysql.user}@${this.mysql.host}:${this.mysql.port}`
                : `${this.mysql.user}@${this.mysql.socket}`;
        this.dbOpened = true;
        this.currentDatabase = '';
        this.activeDatabase = '';
        this.closeConnectionDialog();
        this.resetBrowser();
        document.title = `Sequel Explorer - ${this.connectionLabel}`;
        await this.loadDatabases();

        const lastDatabase = localStorage.getItem('lastMysqlDatabase');
        const database = this.databases.includes(lastDatabase) ? lastDatabase : this.databases[0];
        if (database) await this.selectDatabase(database);
        return true;
    },

    async loadDatabases() {
        const generation = this.viewGeneration;
        try {
            const response = await fetch('/api/databases');
            const data = await response.json();
            if (generation !== this.viewGeneration) return;
            if (data.error) {
                this.browserError = data.error;
                this.databases = [];
                return;
            }
            this.browserError = '';
            this.databases = data;
        } catch (error) {
            if (generation !== this.viewGeneration) return;
            this.browserError = `Failed to load databases: ${error.message}`;
            this.databases = [];
        }
    },

    async selectDatabase(database) {
        if (!database || this.isSelectingDatabase || this.sqlTransferRunning || this.schemaSaving) return;
        this.isSelectingDatabase = true;
        let response;
        try {
            response = await ipcRequest('selectMysqlDatabase', { database });
        } finally {
            this.isSelectingDatabase = false;
        }
        if (!response.ok) {
            this.currentDatabase = this.activeDatabase;
            alert('Failed to open database:\n' + response.error);
            return;
        }
        this.currentDatabase = database;
        this.activeDatabase = database;
        this.dbFileName = database;
        localStorage.setItem('lastMysqlDatabase', database);
        this.resetBrowser();
        document.title = `Sequel Explorer - ${this.connectionLabel} - ${database}`;
        await this.loadTables();
        const lastTable = localStorage.getItem('lastTableName');
        if (lastTable && this.tables.includes(lastTable)) await this.selectTable(lastTable);
    },

    resetBrowser() {
        if (this.$refs.addRowDialog?.open) this.$refs.addRowDialog.close();
        if (this.$refs.usersDialog?.open) this.$refs.usersDialog.close();
        this.viewGeneration += 1;
        this.isLoading = false;
        this.isQueryRunning = false;
        this.tables = [];
        this.currentTable = null;
        this.currentOffset = 0;
        this.currentTotal = 0;
        this.currentCursor = null;
        this.columns = [];
        this.rows = [];
        this.rowCount = '';
        this.schemaSql = '';
        this.schemaTableName = '';
        this.schemaColumns = [];
        this.schemaIndexes = [];
        this.schemaError = '';
        this.schemaLoading = false;
        this.schemaSaving = false;
        this.newSchemaColumn = {
            name: '',
            type: 'TEXT',
            nullable: true,
            defaultSql: '',
            primaryKey: false,
            primaryKeyPosition: 0,
            autoIncrement: false,
            generated: false,
            characterSet: null,
            collation: null,
            comment: '',
            extra: '',
        };
        this.newSchemaIndex = { name: '', columns: [], unique: false, primary: false, readOnly: false };
        this.queryText = '';
        this.isCustomQuery = false;
        this.activeTab = 'data';
        this.showDataTable = false;
        this.showDataLoading = false;
        this.showDataEmpty = false;
        this.browserError = '';
        this.editingCell = null;
        this.cellEditError = '';
        this.rawQueryText = '';
        this.rawQueryColumns = [];
        this.rawQueryRows = [];
        this.rawQueryStatus = 'Ready';
        this.rawQueryError = '';
        this.rawQueryRunning = false;
        this.rawQueryHasRun = false;
        this.newRowFields = [];
        this.addRowError = '';
        this.isAddingRow = false;
        this.rowMenu = null;
        this.isDeletingRow = false;
    },

    async loadTables() {
        const generation = this.viewGeneration;
        try {
            const response = await fetch('/api/tables');
            const data = await response.json();
            if (generation !== this.viewGeneration) return;
            if (data.error) {
                this.browserError = data.error;
                this.tables = [];
                return;
            }
            this.browserError = '';
            this.tables = data;
        } catch (error) {
            if (generation !== this.viewGeneration) return;
            this.browserError = `Failed to load tables: ${error.message}`;
            this.tables = [];
        }
    },

    async selectTable(name) {
        if (name === this.currentTable || this.schemaSaving) return;
        this.currentTable = name;
        this.isCustomQuery = false;
        this.queryText = '';
        localStorage.setItem('lastTableName', name);
        await this.openTableView(name);
    },

    selectTableFromList(name) {
        this.$refs.tablesList.focus({ preventScroll: true });
        this.selectTable(name);
    },

    onTableListKeydown(event) {
        if (!['ArrowUp', 'ArrowDown', 'Home', 'End'].includes(event.key) || this.tables.length === 0) return;
        event.preventDefault();

        const currentIndex = this.tables.indexOf(this.currentTable);
        let targetIndex;
        if (event.key === 'Home') {
            targetIndex = 0;
        } else if (event.key === 'End') {
            targetIndex = this.tables.length - 1;
        } else if (event.key === 'ArrowUp') {
            targetIndex = currentIndex < 0 ? this.tables.length - 1 : Math.max(0, currentIndex - 1);
        } else {
            targetIndex = currentIndex < 0 ? 0 : Math.min(this.tables.length - 1, currentIndex + 1);
        }

        this.selectTable(this.tables[targetIndex]);
        document.getElementById(`table-option-${targetIndex}`)?.scrollIntoView({ block: 'nearest' });
    },

    async openTableView(name) {
        const generation = ++this.viewGeneration;
        document.title = `Sequel Explorer - ${this.dbFileName} - ${name}`;

        this.isLoading = false;
        this.currentOffset = 0;
        this.currentTotal = 0;
        this.currentCursor = null;
        this.isCustomQuery = false;
        this.columns = [];
        this.rows = [];
        this.rowCount = '';
        this.showDataEmpty = false;
        this.showDataLoading = true;
        this.showDataTable = false;
        this.activeTab = 'data';
        this.editingCell = null;
        this.cellEditError = '';
        this.schemaSql = '';
        this.schemaTableName = name;
        this.schemaColumns = [];
        this.schemaIndexes = [];
        this.schemaError = '';
        this.schemaLoading = true;

        await this.loadMoreRows(name, generation);
        if (generation !== this.viewGeneration || name !== this.currentTable) return;

        try {
            const response = await fetch(`/api/table/${encodeURIComponent(name)}/schema`);
            const data = await response.json();
            if (generation !== this.viewGeneration || name !== this.currentTable) return;
            if (data.error) {
                this.schemaError = data.error;
                return;
            }
            this.schemaSql = data.sql || '';
            this.schemaTableName = data.name;
            this.schemaColumns = data.columns.map((column) => ({
                ...column,
                originalName: column.name,
                defaultSql: column.defaultSql || '',
                originalType: column.type,
                originalNullable: column.nullable,
                originalDefaultSql: column.defaultSql || '',
                originalPrimaryKey: column.primaryKey,
                originalAutoIncrement: column.autoIncrement,
            }));
            this.schemaIndexes = data.indexes.map((index) => ({
                ...index,
                originalName: index.name,
                originalUnique: index.unique,
                originalColumns: [...index.columns],
            }));
            this.schemaError = '';
        } catch (error) {
            if (generation !== this.viewGeneration || name !== this.currentTable) return;
            this.schemaError = 'Error loading schema: ' + error.message;
        } finally {
            if (generation === this.viewGeneration && name === this.currentTable) this.schemaLoading = false;
        }
    },

    async loadMoreRows(tableName, generation = this.viewGeneration) {
        if (generation !== this.viewGeneration || tableName !== this.currentTable) return;
        if (this.isLoading) return;
        if (this.currentOffset > 0 && this.currentOffset >= this.currentTotal) return;

        const offset = this.currentOffset;
        this.isLoading = true;
        this.showDataLoading = true;

        try {
            const params = new URLSearchParams({ offset, limit: PAGE_SIZE });
            if (this.currentCursor) params.set('cursor', this.currentCursor);
            const url = `/api/table/${encodeURIComponent(tableName)}/data?${params}`;
            const response = await fetch(url);
            const data = await response.json();
            if (generation !== this.viewGeneration || tableName !== this.currentTable) return;

            if (data.error) {
                this.dataEmptyText = 'Error: ' + data.error;
                this.showDataEmpty = true;
                return;
            }

            if (offset === 0) {
                this.currentTotal = data.total;
                this.rowCount = `${data.total.toLocaleString()} rows`;
                this.columns = data.columns;
                this.showDataTable = true;
                if (data.rows.length === 0) {
                    this.dataEmptyText = 'No rows';
                    this.showDataEmpty = true;
                    return;
                }
            }

            this.appendRows(data.rows);
            this.currentOffset = offset + data.rows.length;
            this.currentCursor = data.next_cursor;
        } catch (error) {
            if (generation !== this.viewGeneration || tableName !== this.currentTable) return;
            this.dataEmptyText = `Failed to load rows: ${error.message}`;
            this.showDataEmpty = true;
        } finally {
            if (generation === this.viewGeneration && tableName === this.currentTable) {
                this.showDataLoading = false;
                this.isLoading = false;
            }
        }
    },

    appendRows(rows) {
        this.rows = this.rows.concat(rows);
    },

    canEditCell(colIdx) {
        return (
            !this.isCustomQuery &&
            this.currentTable !== null &&
            !this.columns[colIdx]?.is_blob &&
            this.columns.some((column) => column.is_primary_key)
        );
    },

    isEditingCell(rowIdx, colIdx) {
        return this.editingCell?.rowIdx === rowIdx && this.editingCell?.colIdx === colIdx;
    },

    startCellEdit(rowIdx, colIdx, value, event) {
        if (!this.canEditCell(colIdx) || this.editingCell?.saving || event.target.closest('button')) return;
        this.cellEditError = '';
        this.editingCell = { rowIdx, colIdx, originalValue: value, saving: false };
        this.editValue = value.kind === 'null' ? 'NULL' : String(value.value);
        requestAnimationFrame(() => {
            const editor = this.$refs.cellEditor;
            if (editor) {
                editor.focus();
                editor.select();
            }
        });
    },

    cancelCellEdit() {
        if (this.editingCell?.saving) return;
        this.editingCell = null;
        this.editValue = '';
    },

    editedCellValue(originalValue, input) {
        if (input.trim().toUpperCase() === 'NULL') return { kind: 'null', value: null };
        if (originalValue.kind === 'integer' && /^[-+]?\d+$/.test(input.trim())) {
            return { kind: 'integer', value: input.trim() };
        }
        if (originalValue.kind === 'float' && input.trim() !== '' && Number.isFinite(Number(input))) {
            return { kind: 'float', value: Number(input) };
        }
        return { kind: 'text', value: input };
    },

    async commitCellEdit() {
        const edit = this.editingCell;
        if (!edit || edit.saving) return;
        const originalText = edit.originalValue.kind === 'null' ? 'NULL' : String(edit.originalValue.value);
        if (this.editValue === originalText) {
            this.cancelCellEdit();
            return;
        }

        edit.saving = true;
        const generation = this.viewGeneration;
        const table = this.currentTable;
        const columnName = this.columns[edit.colIdx].name;
        const isPrimaryKey = this.columns[edit.colIdx].is_primary_key;
        const row = this.rows[edit.rowIdx];
        const keys = Object.fromEntries(
            this.columns
                .map((column, index) => [column, row[index]])
                .filter(([column]) => column.is_primary_key)
                .map(([column, value]) => [column.name, value]),
        );
        const value = this.editedCellValue(edit.originalValue, this.editValue);
        try {
            const response = await fetch(`/api/table/${encodeURIComponent(table)}/data`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ column: columnName, value, keys }),
            });
            const data = await response.json();
            if (generation !== this.viewGeneration || table !== this.currentTable) return;
            if (data.error) {
                this.cellEditError = `Could not save ${columnName}: ${data.error}`;
                return;
            }
            if (isPrimaryKey) {
                await this.openTableView(table);
                return;
            }
            row[edit.colIdx] = value;
            this.rows = [...this.rows];
        } catch (error) {
            if (generation !== this.viewGeneration || table !== this.currentTable) return;
            this.cellEditError = `Could not save ${columnName}: ${error.message}`;
        } finally {
            if (generation === this.viewGeneration && table === this.currentTable && this.editingCell === edit) {
                this.editingCell = null;
                this.editValue = '';
            }
        }
    },

    formatSqlValue(cell) {
        return formatSqlValue(cell);
    },

    quoteIdentifier(identifier) {
        if (this.connectionKind === 'mysql') return '`' + identifier.replace(/`/g, '``') + '`';
        return '"' + identifier.replace(/"/g, '""') + '"';
    },

    async navigateToForeignKey(table, column, cell) {
        this.queryText = `SELECT * FROM ${this.quoteIdentifier(table)} WHERE ${this.quoteIdentifier(column)} = ${this.formatSqlValue(cell)}`;
        await this.runQuery();
    },

    async runQuery() {
        if (this.activeTab === 'query') return this.runRawQuery();
        const sql = this.queryText.trim();
        // Guard against re-entry, the same query can be triggered by the input, the button and a shortcut
        if (!sql || this.isQueryRunning) return;

        const generation = ++this.viewGeneration;
        this.isLoading = false;
        this.isQueryRunning = true;
        this.isCustomQuery = true;
        this.activeTab = 'data';
        this.currentOffset = 0;
        this.currentTotal = 0;
        this.currentCursor = null;
        this.columns = [];
        this.rows = [];
        this.rowCount = '';
        this.showDataEmpty = false;
        this.showDataLoading = true;
        this.showDataTable = false;

        try {
            const response = await fetch('/api/query', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ sql }),
            });
            const data = await response.json();
            if (generation !== this.viewGeneration) return;

            if (data.error) {
                this.dataEmptyText = 'Error: ' + data.error;
                this.showDataEmpty = true;
                return;
            }

            this.rowCount = data.truncated
                ? `${data.rows.length.toLocaleString()}+ rows`
                : `${data.rows.length.toLocaleString()} rows`;
            this.columns = data.columns;
            this.showDataTable = true;

            if (data.rows.length === 0) {
                this.dataEmptyText = 'No rows';
                this.showDataEmpty = true;
                return;
            }

            this.appendRows(data.rows);
        } catch (error) {
            if (generation !== this.viewGeneration) return;
            this.dataEmptyText = `Query failed: ${error.message}`;
            this.showDataEmpty = true;
        } finally {
            if (generation === this.viewGeneration) {
                this.isQueryRunning = false;
                this.showDataLoading = false;
            }
        }
    },

    clearQuery() {
        this.queryText = '';
        this.isCustomQuery = false;
        if (this.currentTable) this.openTableView(this.currentTable);
    },

    onRawQueryKeydown(event) {
        if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
            event.preventDefault();
            this.runRawQuery();
        }
    },

    async runRawQuery() {
        const sql = this.rawQueryText.trim();
        if (!sql || this.rawQueryRunning) return;

        this.rawQueryRunning = true;
        this.rawQueryHasRun = true;
        this.rawQueryError = '';
        this.rawQueryColumns = [];
        this.rawQueryRows = [];
        this.rawQueryStatus = 'Running...';
        const generation = this.viewGeneration;
        const startedAt = performance.now();
        try {
            const response = await fetch('/api/query/raw', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ sql }),
            });
            const data = await response.json();
            if (generation !== this.viewGeneration) return;
            const elapsed = Math.max(1, Math.round(performance.now() - startedAt));
            if (data.error) {
                this.rawQueryError = data.error;
                this.rawQueryStatus = `Failed in ${elapsed} ms`;
                return;
            }

            this.rawQueryColumns = data.columns;
            this.rawQueryRows = data.rows;
            if (data.columns.length > 0) {
                const suffix = data.truncated ? '+' : '';
                this.rawQueryStatus = `${data.rows.length.toLocaleString()}${suffix} rows in ${elapsed} ms`;
            } else if (data.affectedRows > 0) {
                this.rawQueryStatus = `${data.affectedRows.toLocaleString()} rows affected in ${elapsed} ms`;
            } else {
                this.rawQueryStatus = `Statement executed in ${elapsed} ms`;
            }
            await this.loadTables();
        } catch (error) {
            if (generation !== this.viewGeneration) return;
            const elapsed = Math.max(1, Math.round(performance.now() - startedAt));
            this.rawQueryError = error.message;
            this.rawQueryStatus = `Failed in ${elapsed} ms`;
        } finally {
            if (generation === this.viewGeneration) this.rawQueryRunning = false;
        }
    },

    formatRawQueryCell(cell, columnIdx) {
        return formatCellValue(cell, this.rawQueryColumns[columnIdx].is_blob);
    },

    formatCellValue(cell, colIdx) {
        const column = this.columns[colIdx];
        return formatCellValue(cell, column.is_blob);
    },
}).mount('#app');
