/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import { useEffect, useMemo, useState } from 'preact/hooks';
import type { Project, ProjectIndexResponse, Team, TeamIndexResponse } from '../src-gen/api.ts';
import { API_URL, authFetch } from '../services/auth.ts';
import { SidebarLayout, SidebarLink } from './sidebar-layout.tsx';

function AppSidebar() {
    const [teams, setTeams] = useState<Team[]>([]);
    const [projects, setProjects] = useState<Project[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        Promise.all([
            authFetch(`${API_URL}/teams`).then((response) => response.json() as Promise<TeamIndexResponse>),
            authFetch(`${API_URL}/projects`).then((response) => response.json() as Promise<ProjectIndexResponse>),
        ]).then(([teamData, projectData]) => {
            setTeams(teamData.data);
            setProjects(projectData.data);
            setLoading(false);
        });
    }, []);

    const projectsByTeam = useMemo(() => {
        return new Map(teams.map((team) => [team.id, projects.filter((project) => project.teamId === team.id)]));
    }, [projects, teams]);

    return (
        <>
            {loading && <p class="sidebar-empty-text">Loading teams...</p>}
            {!loading &&
                teams.map((team) => {
                    const teamProjects = projectsByTeam.get(team.id) ?? [];

                    return (
                        <div key={team.id} class="sidebar-section">
                            <div class="sidebar-section-title">{team.name}</div>
                            {teamProjects.length === 0 ? (
                                <p class="sidebar-empty-text">No projects</p>
                            ) : (
                                <div class="sidebar-subnav">
                                    {teamProjects.map((project) => (
                                        <SidebarLink
                                            key={project.id}
                                            href={`/projects/${project.id}`}
                                            label={project.name}
                                            class="sidebar-link-project"
                                        >
                                            <svg class="sidebar-link-icon" viewBox="0 0 24 24" fill="currentColor">
                                                <path d="M10 4H4v16h16V8h-8z" />
                                            </svg>
                                        </SidebarLink>
                                    ))}
                                </div>
                            )}
                        </div>
                    );
                })}
        </>
    );
}

export function AppLayout({ children }: { children: ComponentChildren }) {
    return <SidebarLayout sidebar={<AppSidebar />}>{children}</SidebarLayout>;
}
