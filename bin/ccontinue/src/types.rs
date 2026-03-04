/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use indexmap::IndexMap;

use crate::utils::to_snake_case;

#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) type_: String,
    pub(crate) default: Option<String>,
    pub(crate) attributes: IndexMap<String, Vec<String>>,
    pub(crate) class_: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Argument {
    pub(crate) name: String,
    pub(crate) type_: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Method {
    pub(crate) name: String,
    pub(crate) return_type: String,
    pub(crate) is_return_self: bool,
    pub(crate) is_virtual: bool,
    pub(crate) is_static: bool,
    pub(crate) arguments: Vec<Argument>,
    pub(crate) class_: String,
    pub(crate) origin_class: String,
}

#[derive(Debug)]
pub(crate) struct Class {
    pub(crate) name: String,
    pub(crate) snake_name: String,
    pub(crate) parent_name: Option<String>,
    pub(crate) is_abstract: bool,
    pub(crate) fields: IndexMap<String, Field>,
    pub(crate) methods: IndexMap<String, Method>,
    pub(crate) interface_names: Vec<String>,
}

impl Class {
    pub(crate) fn new(name: &str, parent_name: Option<String>) -> Self {
        Class {
            snake_name: to_snake_case(name),
            name: name.to_owned(),
            parent_name,
            is_abstract: false,
            fields: IndexMap::new(),
            methods: IndexMap::new(),
            interface_names: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Interface {
    pub(crate) snake_name: String,
    pub(crate) id: usize,
    pub(crate) parent_names: Vec<String>,
    pub(crate) methods: IndexMap<String, Method>,
    pub(crate) default_bodies: IndexMap<String, String>,
}

impl Interface {
    pub(crate) fn new(name: &str, id: usize) -> Self {
        Interface {
            snake_name: to_snake_case(name),
            id,
            parent_names: Vec::new(),
            methods: IndexMap::new(),
            default_bodies: IndexMap::new(),
        }
    }
}
