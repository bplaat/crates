/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::marker::PhantomData;

use http::{Method, Request, Response};

// MARK: Route
pub type Path = BTreeMap<String, String>;

type Handler<T> = fn(&Request, &T, &Path) -> Response;

#[derive(Debug)]
enum RoutePart {
    Static(String),
    Param(String),
}

struct Route<T> {
    methods: Vec<Method>,
    route: String,
    parts: Vec<RoutePart>,
    handler: Handler<T>,
}

impl<T> Route<T> {
    fn new(methods: Vec<Method>, route: String, handler: Handler<T>) -> Self {
        let parts = route
            .split('/')
            .filter(|part| !part.is_empty())
            .map(|part| {
                if let Some(stripped) = part.strip_prefix(':') {
                    RoutePart::Param(stripped.to_string())
                } else {
                    RoutePart::Static(part.to_string())
                }
            })
            .collect();
        Self {
            methods,
            route,
            parts,
            handler,
        }
    }

    fn matches(&self, path: &str) -> (bool, Option<Path>) {
        let mut path_parts = path.split('/').filter(|part| !part.is_empty());
        let mut params = BTreeMap::new();
        for part in &self.parts {
            match part {
                RoutePart::Static(expected) => {
                    if let Some(actual) = path_parts.next() {
                        if actual != *expected {
                            return (false, None);
                        }
                    } else {
                        return (false, None);
                    }
                }
                RoutePart::Param(name) => {
                    if let Some(actual) = path_parts.next() {
                        params.insert(name.to_string(), actual.to_string());
                    } else {
                        return (false, None);
                    }
                }
            }
        }
        (path_parts.next().is_none(), Some(params))
    }
}

// MARK: Router
pub struct Router<T> {
    routes: Vec<Route<T>>,
    fallback_handler: Option<Handler<T>>,
    _marker: PhantomData<T>,
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

    pub fn next(&self, req: &Request, ctx: &T) -> Response {
        // Match routes
        for route in self.routes.iter().rev() {
            let (matches, path) = route.matches(&req.url.path);
            if matches {
                // Find matching route by method
                for route in self.routes.iter().filter(|r| r.route == route.route) {
                    if !route.methods.contains(&req.method) {
                        continue;
                    }
                    return (route.handler)(req, ctx, &path.unwrap());
                }

                // When method is not allowed
                return Response::new()
                    .status(http::Status::MethodNotAllowed)
                    .body("405 Method Not Allowed");
            }
        }

        // Else run fallback handler
        if let Some(fallback_handler) = self.fallback_handler {
            fallback_handler(req, ctx, &BTreeMap::new())
        } else {
            Response::new()
                .status(http::Status::NotFound)
                .body("404 Not Found")
        }
    }
}
