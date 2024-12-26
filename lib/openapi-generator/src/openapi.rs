/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(missing_docs)]

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenApi {
    // pub openapi: String,
    // pub info: Info,
    // pub servers: Vec<Server>,
    // pub paths: IndexMap<String, PathItem>,
    pub components: Components,
}

// #[derive(Deserialize)]
// pub struct Info {
//     pub title: String,
//     pub description: String,
//     pub version: String,
// }

// #[derive(Deserialize)]
// pub struct Server {
//     pub url: String,
// }

// #[derive(Deserialize)]
// pub struct PathItem {
//     pub get: Option<Operation>,
//     pub post: Option<Operation>,
//     pub put: Option<Operation>,
//     pub delete: Option<Operation>,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Operation {
//     pub tags: Vec<String>,
//     pub summary: String,
//     pub responses: Responses,
//     pub request_body: Option<RequestBody>,
//     pub parameters: Option<Vec<Parameter>>,
// }

// #[derive(Deserialize)]
// pub struct Responses {
//     #[serde(rename = "200")]
//     pub ok: Option<Response>,
//     #[serde(rename = "400")]
//     pub bad_request: Option<Response>,
//     #[serde(rename = "404")]
//     pub not_found: Option<Response>,
// }

// #[derive(Deserialize)]
// pub struct Response {
//     pub description: String,
//     pub content: Option<Content>,
// }

// #[derive(Deserialize)]
// pub struct Content {
//     #[serde(rename = "text/plain")]
//     pub text_plain: Option<ContentSchema>,
//     #[serde(rename = "application/json")]
//     pub application_json: Option<ContentSchema>,
// }

// #[derive(Deserialize)]
// pub struct ContentSchema {
//     pub schema: Schema,
// }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>,
    pub r#type: Option<String>,
    pub format: Option<String>,
    pub properties: Option<IndexMap<String, Schema>>,
    pub additional_properties: Option<Box<Schema>>,
    pub required: Option<Vec<String>>,
    pub items: Option<Box<Schema>>,
    pub r#enum: Option<Vec<String>>,
}

// #[derive(Deserialize)]
// pub struct RequestBody {
//     pub required: bool,
//     pub content: Content,
// }

// #[derive(Deserialize)]
// pub struct Parameter {
//     pub name: String,
//     pub r#in: String,
//     pub required: bool,
//     pub schema: Schema,
// }

#[derive(Deserialize)]
pub struct Components {
    // pub parameters: IndexMap<String, Parameter>,
    pub schemas: IndexMap<String, Schema>,
}
