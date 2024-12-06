/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::marker::PhantomData;

use anyhow::Result;
use http::{Method, Request, Response};
use lazy_static::lazy_static;
use regex::{Captures, Regex};

pub type Path = BTreeMap<String, String>;

type Handler<T> = fn(&Request, &T, &Path) -> Result<Response>;

lazy_static! {
    static ref PATH_PARAM_REGEX: Regex = Regex::new(":([a-z_]+)").expect("Should compile");
}

struct Route<T> {
    methods: Vec<Method>,
    re: Regex,
    handler: Handler<T>,
}

impl<T> Route<T> {
    fn new(methods: Vec<Method>, route: String, handler: Handler<T>) -> Self {
        Self {
            methods,
            re: Regex::new(&format!(
                "^{}$",
                PATH_PARAM_REGEX.replace_all(&route, |captures: &Captures| format!(
                    "(?P<{}>[^/]+)",
                    captures.get(1).unwrap().as_str()
                ))
            ))
            .expect("Invalid route"),
            handler,
        }
    }
}

pub struct Router<T> {
    routes: Vec<Route<T>>,
    fallback_handler: Option<Handler<T>>,
    _marker: PhantomData<T>,
}

impl<T> Router<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(
        mut self,
        methods: &[Method],
        route: impl AsRef<str>,
        handler: Handler<T>,
    ) -> Self {
        self.routes.push(Route::new(
            methods.to_vec(),
            route.as_ref().to_string(),
            handler,
        ));
        self
    }

    pub fn any(self, route: impl AsRef<str>, handler: Handler<T>) -> Self {
        self.route(
            &[Method::Get, Method::Post, Method::Put, Method::Delete],
            route,
            handler,
        )
    }

    pub fn get(self, route: impl AsRef<str>, handler: Handler<T>) -> Self {
        self.route(&[Method::Get], route, handler)
    }

    pub fn post(self, route: impl AsRef<str>, handler: Handler<T>) -> Self {
        self.route(&[Method::Post], route, handler)
    }

    pub fn put(self, route: impl AsRef<str>, handler: Handler<T>) -> Self {
        self.route(&[Method::Put], route, handler)
    }

    pub fn delete(self, route: impl AsRef<str>, handler: Handler<T>) -> Self {
        self.route(&[Method::Delete], route, handler)
    }

    pub fn fallback(mut self, handler: Handler<T>) -> Self {
        self.fallback_handler = Some(handler);
        self
    }

    pub fn next(&self, req: &Request, ctx: &T) -> Result<Response> {
        // Match routes
        for route in self.routes.iter().rev() {
            if route.re.is_match(&req.path) {
                // Check if method is allowed
                if !route.methods.contains(&req.method) {
                    return Ok(Response::new()
                        .status(http::Status::MethodNotAllowed)
                        .body("405 Method Not Allowed"));
                }

                // Get path parameters captured by regex
                let captures = route.re.captures(&req.path).expect("Should be some");
                let mut path = BTreeMap::new();
                for name in route.re.capture_names().flatten() {
                    if let Some(value) = captures.name(name) {
                        path.insert(name.to_string(), value.as_str().to_string());
                    }
                }

                // Run route handler
                return (route.handler)(req, ctx, &path);
            }
        }

        // Else run fallback handler
        if let Some(fallback_handler) = self.fallback_handler {
            fallback_handler(req, ctx, &BTreeMap::new())
        } else {
            Ok(Response::new()
                .status(http::Status::NotFound)
                .body("404 Not Found"))
        }
    }
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Self {
            routes: Vec::new(),
            fallback_handler: None,
            _marker: PhantomData,
        }
    }
}
