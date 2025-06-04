/*
 * Copyright (c) 2023, Bastiaan van der Plaat <bastiaan.v.d.plaat@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

function info(text) {
    console.log(`[INFO] ${text}`);
}

function warn(text) {
    console.error(`[WARN] ${text}`);
}

function error(text) {
    console.error(`[ERROR] ${text}`);
    process.exit(1);
}

export default { info, warn, error };
