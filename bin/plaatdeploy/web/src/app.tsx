/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { LoadingText } from 'plaatui';
import { useEffect } from 'preact/hooks';
import { Route, Switch } from 'wouter-preact';
import { Login } from './pages/auth/login.tsx';
import { Dashboard } from './pages/dashboard.tsx';
import { ProjectPage } from './pages/project.tsx';
import { Settings } from './pages/settings/account.tsx';
import { $authUser, initAuth } from './services/auth.service.ts';

export function App() {
    useEffect(() => {
        void initAuth();
    }, []);
    const user = $authUser.value;
    useEffect(() => {
        const colorScheme = matchMedia('(prefers-color-scheme: dark)');
        const updateTheme = () => {
            const dark = user?.theme === 'dark' || (user?.theme === 'system' && colorScheme.matches);
            document.documentElement.classList.toggle('dark', dark);
        };
        updateTheme();
        colorScheme.addEventListener('change', updateTheme);
        return () => colorScheme.removeEventListener('change', updateTheme);
    }, [user?.theme]);
    if (user === undefined)
        return (
            <div class="centered-screen">
                <LoadingText>Loading</LoadingText>
            </div>
        );
    if (user === null) return <Login />;
    return (
        <Switch>
            <Route path="/" component={Dashboard} />
            <Route path="/projects/:projectId" component={ProjectPage} />
            <Route path="/settings" component={Settings} />
            <Route>
                <Dashboard />
            </Route>
        </Switch>
    );
}
