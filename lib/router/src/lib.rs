/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple router for HTTP library

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

use http::{Method, Request, Response};

// MARK: Handler

/// Parsed path parameters
type HandlerFn<T> = fn(&Request, &T) -> Response;
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

    fn call(&self, req: &Request, ctx: &mut T) -> Response {
        for pre_layer in &self.pre_layers {
            if let Some(mut res) = pre_layer(req, ctx) {
                for post_layer in &self.post_layers {
                    res = post_layer(req, ctx, res);
                }
                return res;
            }
        }
        let mut res = (self.handler)(req, ctx);
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

    fn match_path(&self, path: &str) -> HashMap<String, String> {
        let mut path_parts = path.split('/').filter(|part| !part.is_empty());
        let mut params = HashMap::new();
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

// MARK: RouterBuilder
/// Router builder
pub struct RouterBuilder<T: Clone> {
    ctx: T,
    pre_layers: Vec<PreLayerFn<T>>,
    post_layers: Vec<PostLayerFn<T>>,
    routes: Vec<Route<T>>,
    not_allowed_method_handler: Option<Handler<T>>,
    fallback_handler: Option<Handler<T>>,
}

impl<T: Clone> RouterBuilder<T> {
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
    pub fn route(mut self, methods: &[Method], route: String, handler: HandlerFn<T>) -> Self {
        self.routes.push(Route::new(
            methods.to_vec(),
            route,
            Handler::new(handler, self.pre_layers.clone(), self.post_layers.clone()),
        ));
        self
    }

    /// Add route for any method
    pub fn any(self, route: impl Into<String>, handler: HandlerFn<T>) -> Self {
        self.route(
            &[Method::Get, Method::Post, Method::Put, Method::Delete],
            route.into(),
            handler,
        )
    }

    /// Add route for GET method
    pub fn get(self, route: impl Into<String>, handler: HandlerFn<T>) -> Self {
        self.route(&[Method::Get], route.into(), handler)
    }

    /// Add route for POST method
    pub fn post(self, route: impl Into<String>, handler: HandlerFn<T>) -> Self {
        self.route(&[Method::Post], route.into(), handler)
    }

    /// Add route for PUT method
    pub fn put(self, route: impl Into<String>, handler: HandlerFn<T>) -> Self {
        self.route(&[Method::Put], route.into(), handler)
    }

    /// Add route for DELETE method
    pub fn delete(self, route: impl Into<String>, handler: HandlerFn<T>) -> Self {
        self.route(&[Method::Delete], route.into(), handler)
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

    /// Build router
    pub fn build(self) -> Router<T> {
        Router(Arc::new(InnerRouter {
            ctx: self.ctx,
            routes: self.routes,
            not_allowed_method_handler: self.not_allowed_method_handler.unwrap_or_else(|| {
                Handler::new(
                    |_, _| {
                        Response::new()
                            .status(http::Status::MethodNotAllowed)
                            .body("405 Method Not Allowed")
                    },
                    self.pre_layers.clone(),
                    self.post_layers.clone(),
                )
            }),
            fallback_handler: self.fallback_handler.unwrap_or_else(|| {
                Handler::new(
                    |_, _| {
                        Response::new()
                            .status(http::Status::NotFound)
                            .body("404 Not Found")
                    },
                    self.pre_layers.clone(),
                    self.post_layers.clone(),
                )
            }),
        }))
    }
}

// MARK: InnerRouter
struct InnerRouter<T: Clone> {
    ctx: T,
    routes: Vec<Route<T>>,
    not_allowed_method_handler: Handler<T>,
    fallback_handler: Handler<T>,
}

impl<T: Clone> InnerRouter<T> {
    fn handle(&self, req: &Request) -> Response {
        let mut ctx = self.ctx.clone();

        // Match routes
        for route in self.routes.iter().rev() {
            if route.is_match(&req.url.path) {
                let mut req = req.clone();
                req.params = route.match_path(&req.url.path);

                // Find matching route by method
                for route in self.routes.iter().filter(|r| r.route == route.route) {
                    if route.methods.contains(&req.method) {
                        return route.handler.call(&req, &mut ctx);
                    }
                }

                // Or run not allowed method handler
                return self.not_allowed_method_handler.call(&req, &mut ctx);
            }
        }

        // Or run fallback handler
        self.fallback_handler.call(req, &mut ctx)
    }
}

// MARK: Router
/// Router
#[derive(Clone)]
pub struct Router<T: Clone>(Arc<InnerRouter<T>>);

impl<T: Clone> Router<T> {
    /// Handle request
    pub fn handle(&self, req: &Request) -> Response {
        self.0.handle(req)
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use http::Status;

    use super::*;

    fn home(_req: &Request, _ctx: &()) -> Response {
        Response::new().status(Status::Ok).body("Hello, World!")
    }

    fn hello(req: &Request, _ctx: &()) -> Response {
        let name = req.params.get("name").unwrap();
        Response::new()
            .status(Status::Ok)
            .body(format!("Hello, {}!", name))
    }

    #[test]
    fn test_routing() {
        let router = RouterBuilder::with(())
            .get("/", home)
            .get("/hello/:name", hello)
            .get("/hello/:name/i/:am/so/:deep", hello)
            .build();

        // Test home route
        let res = router.handle(&Request::with_url("http://localhost/"));
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, b"Hello, World!");

        // Test fallback route
        let res = router.handle(&Request::with_url("http://localhost/unknown"));
        assert_eq!(res.status, Status::NotFound);
        assert_eq!(res.body, b"404 Not Found");

        // Test route with params
        let res = router.handle(&Request::with_url("http://localhost/hello/Bassie"));
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, b"Hello, Bassie!");

        // Test route with multiple params
        let res = router.handle(&Request::with_url(
            "http://localhost/hello/Bassie/i/handle/so/much",
        ));
        assert_eq!(res.status, Status::Ok);

        // Test wrong method
        let res = router.handle(&Request::with_url("http://localhost/").method(Method::Options));
        assert_eq!(res.status, Status::MethodNotAllowed);
        assert_eq!(res.body, b"405 Method Not Allowed");
    }
}
