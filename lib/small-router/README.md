# Small-Router Rust library

A simple and small router for the [small-http](../small-http) library.

## Getting Started

A simple example that opens an HTTP server and serves a response:

```rs
use std::net::{Ipv4Addr, TcpListener};
use anyhow::Result;
use small_http::{Request, Response, Status};
use small_router::RouterBuilder;

fn home(_req: &Request, _ctx: &()) -> Result<Response> {
    Ok(Response::with_body("Home"))
}

fn main() {
    let greeting = "Hello".to_string();
    let router = RouterBuilder::new()
        .get("/", home)
        .get("/hello/:name", move |req, _| {
            let name = &req.params["name"];
            Ok(Response::with_body(format!("{greeting}, {name}!")))
        })
        .fallback(|_, _| {
            Ok(Response::with_status(Status::NotFound).body("404 Not Found"))
        })
        .build();

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
        .unwrap_or_else(|_| panic!("Can't bind to port"));
    small_http::serve(listener, move |req| router.handle(req));
}
```

See the [examples](examples/) for many more examples.

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
