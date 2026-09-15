/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import type { SyncStatus } from '../api.ts';
import { formatCompact } from '../format.ts';

export function SyncMonitor() {
    const [sync, setSync] = useState<SyncStatus | null>(null);
    const [connected, setConnected] = useState(false);
    const [expanded, setExpanded] = useState(true);

    useEffect(() => {
        let socket: WebSocket | null = null;
        let reconnect: number | undefined;
        let stopped = false;

        const connect = () => {
            const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
            socket = new WebSocket(`${protocol}//${location.host}/api/sync/stream`);
            socket.addEventListener('open', () => setConnected(true));
            socket.addEventListener('message', (event) => {
                try {
                    setSync(JSON.parse(String(event.data)) as SyncStatus);
                } catch {
                    setConnected(false);
                }
            });
            socket.addEventListener('close', () => {
                setConnected(false);
                if (!stopped) reconnect = window.setTimeout(connect, 3_000);
            });
            socket.addEventListener('error', () => socket?.close());
        };

        fetch('/api/sync')
            .then((response) => response.ok ? response.json() : null)
            .then((status: SyncStatus | null) => status && setSync(status))
            .catch(() => undefined);
        connect();
        return () => {
            stopped = true;
            if (reconnect !== undefined) window.clearTimeout(reconnect);
            socket?.close();
        };
    }, []);

    const isHistory = sync?.phase === 'history';
    const current = isHistory ? sync.historySyncedCount : (sync?.current ?? 0);
    const total = isHistory ? sync.instrumentCount : (sync?.total ?? 0);
    const percentage = total > 0 ? Math.min(100, (current / total) * 100) : 0;
    const complete = sync?.status === 'complete';

    return (
        <aside class={`sync-monitor ${expanded ? 'is-expanded' : ''}`} aria-label="Background data synchronization">
            <button class="sync-monitor-head" type="button" onClick={() => setExpanded(!expanded)} aria-expanded={expanded}>
                <span class={`sync-indicator ${connected ? 'is-connected' : ''} ${complete ? 'is-complete' : ''}`} aria-hidden="true"></span>
                <span>
                    <strong>{complete ? 'Local market mirror ready' : 'Building local market mirror'}</strong>
                    <small>{connected ? 'Live status stream' : 'Reconnecting status stream'}</small>
                </span>
                <em>{expanded ? 'Close' : `${percentage.toFixed(0)}%`}</em>
            </button>
            {expanded && (
                <div class="sync-monitor-body">
                    <div class="sync-progress" role="progressbar" aria-label="Historical data import" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(percentage)}>
                        <span style={{ width: `${percentage}%` }}></span>
                    </div>
                    <div class="sync-progress-copy">
                        <strong>{percentage.toFixed(1)}%</strong>
                        <span>{formatCompact(current)} / {formatCompact(total)} tickers</span>
                    </div>
                    <p>{sync?.message ?? 'Waiting for the background worker...'}</p>
                    <dl>
                        <div><dt>Sync point</dt><dd>{sync?.cursor ?? '--'}</dd></div>
                        <div><dt>Instruments</dt><dd>{formatCompact(sync?.instrumentCount ?? 0)}</dd></div>
                        <div><dt>Price bars</dt><dd>{formatCompact(sync?.storedBarCount ?? 0)}</dd></div>
                        <div><dt>Retries</dt><dd>{formatCompact(sync?.errorCount ?? 0)}</dd></div>
                    </dl>
                </div>
            )}
        </aside>
    );
}

