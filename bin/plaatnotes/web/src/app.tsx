/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect } from 'preact/hooks';
import { Home } from './pages/home.tsx';
import { NotesCreate } from './pages/notes/create.tsx';
import { NotesShow } from './pages/notes/show.tsx';
import { NotFound } from './pages/notfound.tsx';
import { Login } from './pages/auth/login.tsx';
import { Logout } from './pages/auth/logout.tsx';
import { AdminUsers } from './pages/admin/users.tsx';
import { AdminNotes } from './pages/admin/notes.tsx';
import { UserSettings } from './pages/settings.tsx';
import { Route, Router } from './router.tsx';
import { AuthService, $authToken, $isLoading, $authUser } from './services/auth.service.ts';
import { ThemeService } from './services/theme.service.ts';
import { useSignal } from '@preact/signals';

function ProtectedRoute({ children }: { children: any }) {
    const authToken = useSignal($authToken.value);
    const isLoading = useSignal($isLoading.value);

    useEffect(() => {
        const unsubToken = $authToken.subscribe((v) => (authToken.value = v));
        const unsubLoading = $isLoading.subscribe((v) => (isLoading.value = v));
        return () => {
            unsubToken();
            unsubLoading();
        };
    }, []);

    if (isLoading.value) {
        return (
            <div class="container">
                <section class="section">
                    <p>Loading...</p>
                </section>
            </div>
        );
    }

    if (!authToken.value) {
        window.location.href = '/auth/login';
        return null;
    }

    return children;
}

export function App() {
    const authUser = useSignal($authUser.value);

    useEffect(() => {
        AuthService.getInstance().updateAuth();
    }, []);

    useEffect(() => {
        const unsub = $authUser.subscribe((v) => {
            authUser.value = v;
            if (v?.theme) {
                ThemeService.applyTheme(v.theme);
            }
        });
        return unsub;
    }, []);

    useEffect(() => {
        if (authUser.value?.theme) {
            ThemeService.applyTheme(authUser.value.theme);
        }
    }, [authUser.value?.theme]);

    return (
        <Router>
            <Route path="/auth/login" component={Login} />
            <Route path="/auth/logout" component={Logout} />

            <Route
                path="/"
                component={() => (
                    <ProtectedRoute>
                        <Home />
                    </ProtectedRoute>
                )}
            />
            <Route
                path="/notes/create"
                component={() => (
                    <ProtectedRoute>
                        <NotesCreate />
                    </ProtectedRoute>
                )}
            />
            <Route
                path="/notes/:note_id"
                component={({ note_id }: { note_id: string }) => (
                    <ProtectedRoute>
                        <NotesShow note_id={note_id} />
                    </ProtectedRoute>
                )}
            />
            <Route
                path="/admin/users"
                component={() => (
                    <ProtectedRoute>
                        <AdminUsers />
                    </ProtectedRoute>
                )}
            />
            <Route
                path="/admin/notes"
                component={() => (
                    <ProtectedRoute>
                        <AdminNotes />
                    </ProtectedRoute>
                )}
            />
            <Route
                path="/settings"
                component={() => (
                    <ProtectedRoute>
                        <UserSettings />
                    </ProtectedRoute>
                )}
            />
            <Route fallback component={NotFound} />
        </Router>
    );
}
