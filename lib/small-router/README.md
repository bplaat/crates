# Small-Router Rust library

A simple and small router for the [small-http](https://crates.io/crates/small-http) library

## Getting Started

A simple example the opens a http server on serves a simple response:

```rs
use std::net::{Ipv4Addr, TcpListener};
use small_http::{Request, Response, Status};
use small_router::RouterBuilder;

fn home(_req: &Request, _ctx: &()) -> Response {
    Response::with_body("Home")
}
fn about(_req: &Request, _ctx: &()) -> Response {
    Response::with_body("About")
}
fn not_found(_req: &Request, _ctx: &()) -> Response {
    Response::with_status(Status::NotFound).body("404 Not Found")
}

fn main() {
    let router = RouterBuilder::with(())
        .get("/", home)
        .get("/about", about)
        .fallback(not_found)
        .build();

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
        .unwrap_or_else(|_| panic!("Can't bind to port"));
    small_http::serve(listener, move |req| router.handle(req));
}
```

See the [examples](examples/) for many more examples.

## Important: reduce `url` dependencies

## Documentation

See the [documentation](https://docs.rs/small-router) for more information.

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
