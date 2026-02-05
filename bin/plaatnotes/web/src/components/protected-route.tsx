/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { $token } from '../auth.ts';
import { Login } from '../pages/login.tsx';

export function ProtectedRoute({ component: Component }: { component: any }) {
    if (!$token.value) {
        return <Login />;
    }
    return <Component />;
}
