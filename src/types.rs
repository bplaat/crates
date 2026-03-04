// MARK: Types

#![allow(unused)]

use indexmap::IndexMap;

use crate::utils::to_snake_case;

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub type_: String,
    pub default: Option<String>,
    pub attributes: IndexMap<String, Vec<String>>,
    pub class_: String,
}

#[derive(Clone)]
pub struct Argument {
    pub name: String,
    pub type_: String,
}

#[derive(Clone)]
pub struct Method {
    pub name: String,
    pub return_type: String,
    pub is_return_self: bool,
    pub is_virtual: bool,
    pub is_static: bool,
    pub arguments: Vec<Argument>,
    pub class_: String,
    pub origin_class: String,
}

pub struct Class {
    pub name: String,
    pub snake_name: String,
    pub parent_name: Option<String>,
    pub is_abstract: bool,
    pub fields: IndexMap<String, Field>,
    pub methods: IndexMap<String, Method>,
    pub interface_names: Vec<String>,
}

impl Class {
    pub fn new(name: &str, parent_name: Option<String>) -> Self {
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

pub struct Interface {
    pub name: String,
    pub snake_name: String,
    pub id: usize,
    pub parent_names: Vec<String>,
    pub methods: IndexMap<String, Method>,
    pub default_bodies: IndexMap<String, String>,
}

impl Interface {
    pub fn new(name: &str, id: usize) -> Self {
        Interface {
            snake_name: to_snake_case(name),
            name: name.to_owned(),
            id,
            parent_names: Vec::new(),
            methods: IndexMap::new(),
            default_bodies: IndexMap::new(),
        }
    }
}
