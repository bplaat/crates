/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type {
    GithubBranchIndexResponse,
    GithubRepositoryIndexResponse,
    TeamGithubStatusResponse,
} from '../src-gen/api.ts';
import { API_URL, authFetch } from './auth.ts';

export async function loadTeamGithubRepositories(teamId: string): Promise<{
    repositories: string[];
    status: TeamGithubStatusResponse | null;
}> {
    const [statusRes, repositoriesRes] = await Promise.all([
        authFetch(`${API_URL}/teams/${teamId}/github`),
        authFetch(`${API_URL}/teams/${teamId}/github/repositories`),
    ]);

    const status = statusRes.ok ? ((await statusRes.json()) as TeamGithubStatusResponse) : null;
    if (!repositoriesRes.ok) {
        return { repositories: [], status };
    }

    const data: GithubRepositoryIndexResponse = await repositoriesRes.json();
    return {
        repositories: data.data.map((repository) => repository.fullName),
        status,
    };
}

export async function loadTeamGithubBranches(teamId: string, repository: string): Promise<string[]> {
    const res = await authFetch(
        `${API_URL}/teams/${teamId}/github/branches?repository=${encodeURIComponent(repository)}`,
    );
    if (!res.ok) {
        return [];
    }
    const data: GithubBranchIndexResponse = await res.json();
    return data.data.map((branch) => branch.name);
}
