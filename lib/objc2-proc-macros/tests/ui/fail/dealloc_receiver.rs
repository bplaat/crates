/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use objc2_proc_macros::define_class;

define_class!(
    struct Object;

    impl Object {
        #[unsafe(method(dealloc))]
        fn dealloc(&self) {}
    }
);

fn main() {}
