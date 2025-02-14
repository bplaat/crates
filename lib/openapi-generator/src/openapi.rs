/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct OpenApi {
    pub components: Components,
}

#[derive(Deserialize)]
pub(crate) struct Components {
    pub schemas: IndexMap<String, Schema>,
}

#[derive(Deserialize)]
pub(crate) struct Schema {
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>,
    pub r#type: Option<String>,
    pub format: Option<String>,
    pub properties: Option<IndexMap<String, Schema>>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Box<Schema>>,
    pub required: Option<Vec<String>>,
    pub items: Option<Box<Schema>>,
    pub r#enum: Option<Vec<String>>,
}
