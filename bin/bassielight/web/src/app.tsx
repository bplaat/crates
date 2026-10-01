/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { createContext } from 'preact';
import { lazy, Suspense } from 'preact/compat';
import { Route, Switch } from 'wouter-preact';
import { Header } from './components/header.tsx';
import { Ipc } from './ipc.ts';
import { EditorPage } from './pages/editor.tsx';
import { NotFoundPage } from './pages/notfound.tsx';
import { StagePage } from './pages/stage.tsx';
import { initStageStore } from './stage.ts';

export const IpcContext = createContext<Ipc | null>(null);

// The scripts page brings the Monaco code editor, so it only loads when opened
const ScriptsPage = lazy(() => import('./pages/scripts.tsx').then(({ ScriptsPage }) => ({ default: ScriptsPage })));

const ipc = new Ipc();
initStageStore(ipc);

export function App() {
    return (
        <IpcContext.Provider value={ipc}>
            <Header />

            <div class="content">
                <Suspense fallback={null}>
                    <Switch>
                        <Route path="/" component={StagePage} />
                        <Route path="/editor" component={EditorPage} />
                        <Route path="/scripts" component={ScriptsPage} />
                        <Route component={NotFoundPage} />
                    </Switch>
                </Suspense>
            </div>
        </IpcContext.Provider>
    );
}
