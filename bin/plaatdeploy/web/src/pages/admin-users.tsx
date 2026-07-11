/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { AdminLayout } from '../components/admin-layout.tsx';
import { UsersPage } from './users.tsx';

export function AdminUsersPage() {
    return (
        <AdminLayout>
            <UsersPage />
        </AdminLayout>
    );
}
