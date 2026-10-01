/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { createContext } from 'preact';
import { Route, Switch } from 'wouter-preact';
import { Header } from './components/header.tsx';
import { Ipc } from './ipc.ts';
import { EditorPage } from './pages/editor.tsx';
import { NotFoundPage } from './pages/notfound.tsx';
import { StagePage } from './pages/stage.tsx';
import { initStageStore } from './stage.ts';

export const IpcContext = createContext<Ipc | null>(null);

const ipc = new Ipc();
initStageStore(ipc);

export function App() {
    return (
        <IpcContext.Provider value={ipc}>
            <Header />

            <div class="content">
                <Switch>
                    <Route path="/" component={StagePage} />
                    <Route path="/editor" component={EditorPage} />
                    <Route component={NotFoundPage} />
                </Switch>
            </div>
        </IpcContext.Provider>
    );
}
