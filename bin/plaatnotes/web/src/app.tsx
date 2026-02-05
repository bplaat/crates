/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Home } from './pages/home.tsx';
import { Login } from './pages/login.tsx';
import { Settings } from './pages/settings.tsx';
import { NotesCreate } from './pages/notes/create.tsx';
import { NotesShow } from './pages/notes/show.tsx';
import { NotFound } from './pages/notfound.tsx';
import { Route, Router } from './router.tsx';
import { Nav } from './components/nav.tsx';
import { $token } from './auth.ts';

export function App() {
    return (
        <Router>
            <Nav />
            <Route path="/login" component={Login} />
            {$token.value && (
                <>
                    <Route path="/" component={Home} />
                    <Route path="/notes/create" component={NotesCreate} />
                    <Route path="/notes/:note_id" component={NotesShow} />
                    <Route path="/settings" component={Settings} />
                </>
            )}
            <Route fallback component={NotFound} />
        </Router>
    );
}
