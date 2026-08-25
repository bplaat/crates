/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

window.addEventListener('contextmenu', (event) => event.preventDefault());

const nameCollator = new Intl.Collator(undefined, {
    numeric: true,
    sensitivity: 'base',
});

function ipcSend(type, data = {}) {
    window.ipc.postMessage(JSON.stringify({ type, ...data }));
}

PetiteVue.createApp({
    catalog: null,
    items: [],
    activeGroup: 'all',
    search: '',
    operation: null,
    cancelling: false,
    progressIndex: 0,
    progressTotal: 0,
    currentCleanupId: '',
    hasScanned: false,
    showConfirm: false,
    elevationError: '',
    elevating: false,
    appError: '',
    completionMessage: '',
    lastRecovered: null,
    lastSkipped: 0,
    diskFree: null,
    isAdministrator: false,

    init() {
        window.ipc.addEventListener('message', (event) => this.handleMessage(JSON.parse(event.data)));
        ipcSend('initialize');
    },

    handleMessage(message) {
        switch (message.type) {
            case 'catalog':
                this.catalog = message.catalog;
                this.diskFree = message.diskFree;
                this.isAdministrator = message.isAdministrator;
                this.items = message.catalog.groups.flatMap((group) =>
                    group.rules.map((cleanup) => ({ ...cleanup, groupId: group.id, selected: false, result: null })),
                );
                break;
            case 'operationStarted':
                this.operation = message.operation;
                this.cancelling = false;
                this.progressIndex = 0;
                this.progressTotal = message.total;
                if (message.operation === 'clean') {
                    this.lastRecovered = 0;
                    this.lastSkipped = 0;
                }
                break;
            case 'cleanupProgress':
                this.currentCleanupId = message.cleanupId;
                this.progressIndex = message.index + 1;
                this.progressTotal = message.total;
                break;
            case 'scanResult': {
                const item = this.items.find((candidate) => candidate.id === message.result.id);
                if (item) {
                    item.result = message.result;
                    const hasWork = message.result.bytes > 0 || message.result.files > 0 || message.result.unknownSize;
                    item.selected = message.result.available && hasWork && !this.cleanDisabled(item);
                }
                break;
            }
            case 'scanFinished':
                this.operation = null;
                this.cancelling = false;
                this.currentCleanupId = '';
                this.hasScanned = !message.cancelled;
                this.completionMessage = message.cancelled
                    ? 'Scan cancelled'
                    : `Scan complete · ${this.formatBytes(this.totalBytes)} found`;
                break;
            case 'cleanResult': {
                const item = this.items.find((candidate) => candidate.id === message.result.id);
                if (item) {
                    const previousBytes = item.result?.bytes || 0;
                    const previousFiles = item.result?.files || 0;
                    item.result = {
                        ...item.result,
                        ...message.result,
                        bytes: Math.max(0, previousBytes - message.result.cleanedBytes),
                        files: Math.max(0, previousFiles - message.result.cleanedFiles),
                    };
                    item.selected = false;
                }
                this.lastRecovered += message.result.cleanedBytes;
                this.lastSkipped += message.result.skipped;
                break;
            }
            case 'cleanFinished':
                this.operation = null;
                this.currentCleanupId = '';
                this.lastRecovered = message.recoveredBytes;
                this.completionMessage = `Cleanup complete · ${this.formatBytes(this.lastRecovered)} recovered${
                    this.lastSkipped ? ` · ${this.lastSkipped.toLocaleString()} skipped` : ''
                }`;
                break;
            case 'diskFreeUpdated':
                if (message.diskFree !== null) this.diskFree = message.diskFree;
                break;
            case 'fatalError':
                this.operation = null;
                this.cancelling = false;
                this.appError = message.message;
                break;
            case 'elevationError':
                this.elevating = false;
                this.elevationError = message.message;
                break;
        }
    },

    startScan() {
        this.completionMessage = '';
        this.hasScanned = false;
        this.operation = 'scan';
        this.cancelling = false;
        this.progressIndex = 0;
        this.progressTotal = this.items.length;
        this.currentCleanupId = '';
        for (const item of this.items) {
            item.result = null;
            item.selected = false;
        }
        ipcSend('startScan');
    },

    cancelScan() {
        if (this.cancelling) return;
        this.cancelling = true;
        ipcSend('cancelScan');
    },

    restartElevated() {
        if (this.elevating) return;
        this.elevating = true;
        this.elevationError = '';
        ipcSend('restartElevated');
    },

    startClean() {
        const cleanupIds = this.selectedItems.map((item) => item.id);
        this.showConfirm = false;
        this.completionMessage = '';
        this.operation = 'clean';
        this.progressIndex = 0;
        this.progressTotal = cleanupIds.length;
        this.currentCleanupId = cleanupIds[0] || '';
        ipcSend('startClean', { cleanupIds });
    },

    toggleVisible(event) {
        for (const item of this.filteredItems) {
            if (!this.cleanDisabled(item)) item.selected = event.target.checked;
        }
    },

    groupName(id) {
        return this.catalog?.groups.find((group) => group.id === id)?.name || '';
    },

    formatBytes(bytes) {
        if (!bytes) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const unit = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
        const value = bytes / 1024 ** unit;
        return `${value >= 10 || unit === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unit]}`;
    },

    get busy() {
        return this.operation !== null;
    },

    get availableItems() {
        return this.items.filter((item) => item.result?.available);
    },

    get visibleGroups() {
        if (!this.catalog) return [];
        return this.catalog.groups
            .map((group) => ({
                ...group,
                availableCount: this.availableItems.filter((item) => item.groupId === group.id).length,
            }))
            .filter((group) => group.availableCount > 0)
            .sort((left, right) => nameCollator.compare(left.name, right.name));
    },

    get filteredItems() {
        const query = this.search.trim().toLowerCase();
        return this.availableItems
            .filter(
                (item) =>
                    (this.activeGroup === 'all' || item.groupId === this.activeGroup) &&
                    (!query ||
                        item.name.toLowerCase().includes(query) ||
                        item.description.toLowerCase().includes(query)),
            )
            .sort((left, right) => {
                if (this.activeGroup === 'all') {
                    const groupOrder = nameCollator.compare(
                        this.groupName(left.groupId),
                        this.groupName(right.groupId),
                    );
                    if (groupOrder !== 0) return groupOrder;
                }
                return nameCollator.compare(left.name, right.name);
            });
    },

    get selectedItems() {
        return this.availableItems.filter((item) => item.selected && !this.cleanDisabled(item));
    },

    get selectedBytes() {
        return this.selectedItems.reduce((total, item) => total + (item.result?.bytes || 0), 0);
    },

    get selectedHighImpactItems() {
        return this.selectedItems.filter((item) => item.impact === 'high');
    },

    get selectedRedownloadItems() {
        return this.selectedItems.filter((item) => item.recovery === 'redownload');
    },

    get totalBytes() {
        return this.availableItems.reduce((total, item) => total + (item.result?.bytes || 0), 0);
    },

    get totalFiles() {
        return this.availableItems.reduce((total, item) => total + (item.result?.files || 0), 0);
    },

    get allVisibleSelected() {
        const selectable = this.filteredItems.filter((item) => !this.cleanDisabled(item));
        return selectable.length > 0 && selectable.every((item) => item.selected);
    },

    get progressPercent() {
        return this.progressTotal ? Math.min(100, (this.progressIndex / this.progressTotal) * 100) : 0;
    },

    get currentCleanupName() {
        return this.items.find((item) => item.id === this.currentCleanupId)?.name || 'system';
    },

    get pageTitle() {
        if (this.activeGroup === 'all') return 'System cleanup';
        return this.groupName(this.activeGroup);
    },

    get statusText() {
        if (this.operation === 'scan') return 'Looking for files that can be safely regenerated';
        if (this.operation === 'clean') return 'Removing only the categories you confirmed';
        if (this.hasScanned) return `${this.availableItems.length} cleanup categories detected`;
        return 'Review before you remove anything';
    },

    cleanDisabled(item) {
        return item.requiresAdministrator && !this.isAdministrator;
    },
}).mount('#app');
