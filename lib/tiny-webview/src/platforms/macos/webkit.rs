/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[link(name = "WebKit", kind = "framework")]
unsafe extern "C" {}

pub(crate) const WK_NAVIGATION_ACTION_POLICY_CANCEL: i64 = 0;
pub(crate) const WK_NAVIGATION_ACTION_POLICY_ALLOW: i64 = 1;

#[cfg(feature = "ipc")]
pub(crate) const WK_USER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_START: i64 = 0;
