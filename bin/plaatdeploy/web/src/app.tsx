/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Route, Switch } from 'wouter-preact';
import { useEffect } from 'preact/hooks';
import { AdminTeamsPage } from './pages/admin-teams.tsx';
import { AdminUsersPage } from './pages/admin-users.tsx';
import { DashboardPage } from './pages/dashboard.tsx';
import { DeploymentPage } from './pages/deployment.tsx';
import { LoginPage } from './pages/login.tsx';
import { NotFoundPage } from './pages/notfound.tsx';
import { ProjectPage } from './pages/project.tsx';
import { SettingsAccountPage } from './pages/settings-account.tsx';
import { SettingsSessionsPage } from './pages/settings-sessions.tsx';
import { TeamsPage } from './pages/teams.tsx';
import { $authUser, initAuth } from './services/auth.ts';

export function App() {
    useEffect(() => {
        initAuth();
    }, []);

    const authUser = $authUser.value;

    if (authUser === undefined) {
        return <div class="loading">Loading...</div>;
    }

    if (authUser === null) {
        return <LoginPage />;
    }

    return (
        <Switch>
            <Route path="/" component={DashboardPage} />
            <Route path="/admin/users" component={AdminUsersPage} />
            <Route path="/admin/teams" component={AdminTeamsPage} />
            <Route path="/teams" component={TeamsPage} />
            <Route path="/projects/:id" component={ProjectPage} />
            <Route path="/deployments/:id" component={DeploymentPage} />
            <Route path="/settings" component={SettingsAccountPage} />
            <Route path="/settings/sessions" component={SettingsSessionsPage} />
            <Route path="/users" component={AdminUsersPage} />
            <Route component={NotFoundPage} />
        </Switch>
    );
}
