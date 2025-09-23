/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Home } from './pages/home.tsx';
import { NotesCreate } from './pages/notes/create.tsx';
import { NotesShow } from './pages/notes/show.tsx';
import { NotFound } from './pages/notfound.tsx';
import { Route, Router } from './router.tsx';

export function App() {
    return (
        <Router>
            <Route path="/" component={Home} />
            <Route path="/notes/create" component={NotesCreate} />
            <Route path="/notes/:note_id" component={NotesShow} />
            <Route fallback component={NotFound} />
        </Router>
    );
}
