/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(dead_code)]

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct OpenApi {
    pub(crate) openapi: String,
    pub(crate) info: Info,
    pub(crate) servers: Vec<Server>,
    pub(crate) paths: IndexMap<String, PathItem>,
    pub(crate) components: Option<Components>,
}

#[derive(Deserialize)]
pub(crate) struct Info {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) version: String,
}

#[derive(Deserialize)]
pub(crate) struct Server {
    pub(crate) url: String,
}

#[derive(Deserialize)]
pub(crate) struct PathItem {
    pub(crate) get: Option<Operation>,
    pub(crate) post: Option<Operation>,
    pub(crate) put: Option<Operation>,
    pub(crate) delete: Option<Operation>,
}

#[derive(Deserialize)]
pub(crate) struct Operation {
    pub(crate) tags: Vec<String>,
    pub(crate) summary: String,
    pub(crate) responses: Responses,
    #[serde(rename = "requestBody")]
    pub(crate) request_body: Option<RequestBody>,
    pub(crate) parameters: Option<Vec<Parameter>>,
}

#[derive(Deserialize)]
pub(crate) struct Responses {
    #[serde(rename = "200")]
    pub(crate) ok: Option<Response>,
    #[serde(rename = "400")]
    pub(crate) bad_request: Option<Response>,
    #[serde(rename = "404")]
    pub(crate) not_found: Option<Response>,
}

#[derive(Deserialize)]
pub(crate) struct Response {
    pub(crate) description: String,
    pub(crate) content: Option<Content>,
}

#[derive(Deserialize)]
pub(crate) struct Content {
    #[serde(rename = "text/plain")]
    pub(crate) text_plain: Option<ContentSchema>,
    #[serde(rename = "application/json")]
    pub(crate) application_json: Option<ContentSchema>,
}

#[derive(Deserialize)]
pub(crate) struct ContentSchema {
    pub(crate) schema: Schema,
}

#[derive(Deserialize)]
pub(crate) struct Schema {
    pub(crate) r#type: String,
    pub(crate) format: Option<String>,
    pub(crate) properties: Option<IndexMap<String, Schema>>,
    #[serde(rename = "additionalProperties")]
    pub(crate) additional_properties: Option<Box<Schema>>,
    pub(crate) required: Option<Vec<String>>,
    pub(crate) items: Option<Box<Schema>>,
    pub(crate) r#enum: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct RequestBody {
    pub(crate) required: bool,
    pub(crate) content: Content,
}

#[derive(Deserialize)]
pub(crate) struct Parameter {
    pub(crate) name: String,
    #[serde(rename = "in")]
    pub(crate) location: String,
    pub(crate) required: bool,
    pub(crate) schema: Schema,
}

#[derive(Deserialize)]
pub(crate) struct Components {
    pub(crate) parameters: IndexMap<String, Parameter>,
    pub(crate) schemas: IndexMap<String, Schema>,
}
