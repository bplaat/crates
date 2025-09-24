/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { createContext } from 'preact';
import { Route, Router } from './router.tsx';
import { StagePage } from './pages/stage.tsx';
import { SettingsPage } from './pages/settings.tsx';
import { NotFoundPage } from './pages/notfound.tsx';
import { Menubar } from './components/menubar.tsx';
import { EditorPage } from './pages/editor.tsx';
import { Ipc } from './ipc.ts';

export const IpcContext = createContext<Ipc | null>(null);

export function App() {
    return (
        <IpcContext.Provider value={new Ipc()}>
            <Menubar />

            <Router>
                <Route path="/" component={StagePage} />
                <Route path="/editor" component={EditorPage} />
                <Route path="/settings" component={SettingsPage} />
                <Route fallback component={NotFoundPage} />
            </Router>
        </IpcContext.Provider>
    );
}
