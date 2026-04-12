/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Link } from 'wouter-preact';
import { AdminLayout } from '../components/admin-layout.tsx';
import { AppLayout } from '../components/app-layout.tsx';

export function NotFoundPage() {
    const content = (
        <div class="page">
            <div class="empty">
                <h2>404 Not Found</h2>
                <p>
                    <Link href="/">Go back home</Link>
                </p>
            </div>
        </div>
    );

    if (window.location.pathname.startsWith('/admin/')) {
        return <AdminLayout>{content}</AdminLayout>;
    }

    return <AppLayout>{content}</AppLayout>;
}
