/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use objc2_proc_macros::{define_class, extern_class};

extern_class!(struct Generic<T>;);

define_class!(struct Fields { value: usize });

fn main() {}
