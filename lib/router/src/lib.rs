/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple router for HTTP library

use std::collections::BTreeMap;
use std::marker::PhantomData;

use http::{Method, Request, Response};

// MARK: Handler

/// Parsed path parameters
pub type Path = BTreeMap<String, String>;

type HandlerFn<T> = fn(&Request, &T, &Path) -> Response;
type PreLayerFn = fn(&Request) -> Option<Response>;
type PostLayerFn = fn(&Request, Response) -> Response;

struct Handler<T> {
    handler: HandlerFn<T>,
    pre_layers: Vec<PreLayerFn>,
    post_layers: Vec<PostLayerFn>,
}

impl<T> Handler<T> {
    fn new(
        handler: HandlerFn<T>,
        pre_layers: Vec<PreLayerFn>,
        post_layers: Vec<PostLayerFn>,
    ) -> Self {
        Self {
            handler,
            pre_layers,
            post_layers,
        }
    }

    fn call(&self, req: &Request, ctx: &T, path: &Path) -> Response {
        for pre_layer in &self.pre_layers {
            if let Some(res) = pre_layer(req) {
                return res;
            }
        }
        let mut res = (self.handler)(req, ctx, path);
        for post_layer in &self.post_layers {
            res = post_layer(req, res);
        }
        res
    }
}

// MARK: Route
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
        let parts = Self::route_parse_parts(&route);
        Self {
            methods,
            route,
            parts,
            handler,
        }
    }

    fn route_parse_parts(route: &str) -> Vec<RoutePart> {
        route
            .split('/')
            .filter(|part| !part.is_empty())
            .map(|part| {
                if let Some(stripped) = part.strip_prefix(':') {
                    RoutePart::Param(stripped.to_string())
                } else {
                    RoutePart::Static(part.to_string())
                }
            })
            .collect()
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
/// Router
pub struct Router<T> {
    pre_layers: Vec<PreLayerFn>,
    post_layers: Vec<PostLayerFn>,
    routes: Vec<Route<T>>,
    fallback_handler: Option<Handler<T>>,
    _marker: PhantomData<T>,
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Self {
            pre_layers: Vec::new(),
            post_layers: Vec::new(),
            routes: Vec::new(),
            fallback_handler: None,
            _marker: PhantomData,
        }
    }
}

impl<T> Router<T> {
    /// Create new router
    pub fn new() -> Self {
        Self::default()
    }

    /// Add pre layer
    pub fn pre_layer(mut self, layer: PreLayerFn) -> Self {
        self.pre_layers.push(layer);
        self
    }

    /// Add post layer
    pub fn post_layer(mut self, layer: PostLayerFn) -> Self {
        self.post_layers.push(layer);
        self
    }

    /// Add route
    pub fn route(
        mut self,
        methods: &[Method],
        route: impl AsRef<str>,
        handler: HandlerFn<T>,
    ) -> Self {
        self.routes.push(Route::new(
            methods.to_vec(),
            route.as_ref().to_string(),
            Handler::new(handler, self.pre_layers.clone(), self.post_layers.clone()),
        ));
        self
    }

    /// Add route for any method
    pub fn any(self, route: impl AsRef<str>, handler: HandlerFn<T>) -> Self {
        self.route(
            &[Method::Get, Method::Post, Method::Put, Method::Delete],
            route,
            handler,
        )
    }

    /// Add route for GET method
    pub fn get(self, route: impl AsRef<str>, handler: HandlerFn<T>) -> Self {
        self.route(&[Method::Get], route, handler)
    }

    /// Add route for POST method
    pub fn post(self, route: impl AsRef<str>, handler: HandlerFn<T>) -> Self {
        self.route(&[Method::Post], route, handler)
    }

    /// Add route for PUT method
    pub fn put(self, route: impl AsRef<str>, handler: HandlerFn<T>) -> Self {
        self.route(&[Method::Put], route, handler)
    }

    /// Add route for DELETE method
    pub fn delete(self, route: impl AsRef<str>, handler: HandlerFn<T>) -> Self {
        self.route(&[Method::Delete], route, handler)
    }

    /// Set fallback handler
    pub fn fallback(mut self, handler: HandlerFn<T>) -> Self {
        self.fallback_handler = Some(Handler::new(
            handler,
            self.pre_layers.clone(),
            self.post_layers.clone(),
        ));
        self
    }

    /// Handle request
    pub fn handle(&self, req: &Request, ctx: &T) -> Response {
        // Match routes
        for route in self.routes.iter().rev() {
            let (matches, path) = route.matches(&req.url.path);
            if matches {
                // Find matching route by method
                for route in self.routes.iter().filter(|r| r.route == route.route) {
                    if !route.methods.contains(&req.method) {
                        continue;
                    }
                    return route.handler.call(req, ctx, &path.unwrap());
                }

                // When method is not allowed
                return Response::new()
                    .status(http::Status::MethodNotAllowed)
                    .body("405 Method Not Allowed");
            }
        }

        // Else run fallback handler
        if let Some(fallback_handler) = &self.fallback_handler {
            fallback_handler.call(req, ctx, &BTreeMap::new())
        } else {
            Response::new()
                .status(http::Status::NotFound)
                .body("404 Not Found")
        }
    }
}
