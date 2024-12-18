/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple router for HTTP library

use std::collections::BTreeMap;
use std::sync::Arc;

use http::{Method, Request, Response};

// MARK: Handler

/// Parsed path parameters
pub type Path = BTreeMap<String, String>;

type HandlerFn<T> = fn(&Request, &T, &Path) -> Response;
type PreLayerFn<T> = fn(&Request, &mut T) -> Option<Response>;
type PostLayerFn<T> = fn(&Request, &mut T, Response) -> Response;

struct Handler<T> {
    handler: HandlerFn<T>,
    pre_layers: Vec<PreLayerFn<T>>,
    post_layers: Vec<PostLayerFn<T>>,
}

impl<T> Handler<T> {
    fn new(
        handler: HandlerFn<T>,
        pre_layers: Vec<PreLayerFn<T>>,
        post_layers: Vec<PostLayerFn<T>>,
    ) -> Self {
        Self {
            handler,
            pre_layers,
            post_layers,
        }
    }

    fn call(&self, req: &Request, ctx: &mut T, path: &Path) -> Response {
        for pre_layer in &self.pre_layers {
            if let Some(mut res) = pre_layer(req, ctx) {
                for post_layer in &self.post_layers {
                    res = post_layer(req, ctx, res);
                }
                return res;
            }
        }
        let mut res = (self.handler)(req, ctx, path);
        for post_layer in &self.post_layers {
            res = post_layer(req, ctx, res);
        }
        res
    }
}

// MARK: Route
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

    fn is_match(&self, path: &str) -> bool {
        let mut path_parts = path.split('/').filter(|part| !part.is_empty());
        for part in &self.parts {
            match part {
                RoutePart::Static(expected) => {
                    if let Some(actual) = path_parts.next() {
                        if actual != *expected {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                RoutePart::Param(_) => {
                    if path_parts.next().is_none() {
                        return false;
                    }
                }
            }
        }
        path_parts.next().is_none()
    }

    fn match_path(&self, path: &str) -> Path {
        let mut path_parts = path.split('/').filter(|part| !part.is_empty());
        let mut params = BTreeMap::new();
        for part in &self.parts {
            match part {
                RoutePart::Static(_) => {
                    path_parts.next();
                }
                RoutePart::Param(name) => {
                    if let Some(value) = path_parts.next() {
                        params.insert(name.clone(), value.to_string());
                    }
                }
            }
        }
        params
    }
}

// MARK: Router
/// Router
pub struct Router<T: Clone> {
    ctx: T,
    pre_layers: Vec<PreLayerFn<T>>,
    post_layers: Vec<PostLayerFn<T>>,
    routes: Vec<Route<T>>,
    not_allowed_method_handler: Option<Handler<T>>,
    fallback_handler: Option<Handler<T>>,
}

impl<T: Clone> Router<T> {
    /// Create new router with context
    pub fn with(ctx: T) -> Self {
        Self {
            ctx,
            pre_layers: Vec::new(),
            post_layers: Vec::new(),
            routes: Vec::new(),
            not_allowed_method_handler: None,
            fallback_handler: None,
        }
    }

    /// Complete building router
    pub fn build(mut self) -> Arc<Self> {
        // Set default handlers
        if self.fallback_handler.is_none() {
            self.fallback_handler = Some(Handler::new(
                |_, _, _| {
                    Response::new()
                        .status(http::Status::NotFound)
                        .body("404 Not Found")
                },
                self.pre_layers.clone(),
                self.post_layers.clone(),
            ));
        }
        self.not_allowed_method_handler = Some(Handler::new(
            |_, _, _| {
                Response::new()
                    .status(http::Status::MethodNotAllowed)
                    .body("405 Method Not Allowed")
            },
            self.pre_layers.clone(),
            self.post_layers.clone(),
        ));

        Arc::new(self)
    }

    /// Add pre layer
    pub fn pre_layer(mut self, layer: PreLayerFn<T>) -> Self {
        self.pre_layers.push(layer);
        self
    }

    /// Add post layer
    pub fn post_layer(mut self, layer: PostLayerFn<T>) -> Self {
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
    pub fn handle(&self, req: &Request) -> Response {
        let mut ctx = self.ctx.clone();

        // Match routes
        for route in self.routes.iter().rev() {
            if route.is_match(&req.url.path) {
                let path = route.match_path(&req.url.path);

                // Find matching route by method
                for route in self.routes.iter().filter(|r| r.route == route.route) {
                    if !route.methods.contains(&req.method) {
                        continue;
                    }
                    return route.handler.call(req, &mut ctx, &path);
                }

                // Else run not allowed method handler
                return self
                    .not_allowed_method_handler
                    .as_ref()
                    .expect("Should be some")
                    .call(req, &mut ctx, &path);
            }
        }

        // Else run fallback handler
        self.fallback_handler
            .as_ref()
            .expect("Should be some")
            .call(req, &mut ctx, &BTreeMap::new())
    }
}
