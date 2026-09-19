/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    type Deployment,
    type DeploymentMode,
    type GitHubRepository,
    type GitHubRepositoryScan,
    type Project,
} from '../../src-gen/api.ts';
import { authFetch } from './auth.service.ts';

export async function listProjects(): Promise<Project[]> {
    const response = await authFetch('/api/projects');
    if (!response.ok) throw new Error(`Could not load projects (${response.status})`);
    return response.json();
}

export async function getProject(id: string): Promise<Project | null> {
    const response = await authFetch(`/api/projects/${id}`);
    if (response.status === 404) return null;
    if (!response.ok) throw new Error(`Could not load project (${response.status})`);
    return response.json();
}

export async function listRepositories(): Promise<GitHubRepository[]> {
    const response = await authFetch('/api/github/repositories');
    if (!response.ok) throw new Error(`Could not load repositories (${response.status})`);
    return response.json();
}

export async function scanRepository(githubRepo: string): Promise<GitHubRepositoryScan> {
    const response = await authFetch('/api/github/repositories/scan', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ githubRepo }),
    });
    if (!response.ok) throw new Error(`Could not scan repository (${response.status})`);
    return response.json();
}

export async function createProject(
    githubRepo: string,
    deploymentMode: DeploymentMode,
    dockerfilePath?: string,
): Promise<Project> {
    const response = await authFetch('/api/projects', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ githubRepo, deploymentMode, dockerfilePath }),
    });
    if (!response.ok) throw new Error(`Could not create project (${response.status})`);
    return response.json();
}

export async function deleteProject(id: string) {
    return (await authFetch(`/api/projects/${id}`, { method: 'DELETE' })).ok;
}

export async function deployProject(id: string) {
    return (await authFetch(`/api/projects/${id}/deployments`, { method: 'POST' })).ok;
}

export async function listDeployments(id: string): Promise<Deployment[]> {
    const response = await authFetch(`/api/projects/${id}/deployments`);
    if (!response.ok) throw new Error(`Could not load deployments (${response.status})`);
    return response.json();
}
