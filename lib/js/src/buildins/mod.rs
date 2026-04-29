/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::value::Value;

mod globals;

pub(crate) fn env() -> Rc<RefCell<IndexMap<String, Value>>> {
    let env = Rc::new(RefCell::new(IndexMap::new()));
    globals::extend(&env);
    env
}
