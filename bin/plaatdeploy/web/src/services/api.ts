/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export async function jsonOrThrow<T>(res: Response): Promise<T> {
    if (!res.ok) {
        throw new Error(`Request failed with status ${res.status}`);
    }
    return (await res.json()) as T;
}
