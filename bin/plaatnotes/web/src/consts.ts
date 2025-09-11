/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export const API_URL = import.meta.env.MODE === 'release' ? '/api' : 'http://localhost:8080/api';
