/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Native Quick Look framework declaration.

use objc2::extern_class;
#[allow(unused_imports)]
use objc2::runtime::NSObject;

extern_class!(
    #[unsafe(super(NSObject))]
    pub(crate) struct NSViewController;
);

#[link(name = "QuickLookUI", kind = "framework")]
unsafe extern "C" {}
