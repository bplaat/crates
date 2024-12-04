/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::net::TcpListener;

use threadpool::ThreadPool;

pub use crate::method::Method;
pub use crate::request::Request;
pub use crate::response::Response;

mod method;
mod request;
mod response;

// MARK: Status
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Ok = 200,
    TemporaryRedirect = 307,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
}

// MARK: Serve
pub fn serve<F>(listener: TcpListener, handler: F)
where
    F: Fn(&Request) -> Response + Clone + Send + Sync + 'static,
{
    let pool = ThreadPool::new(16);
    for mut stream in listener.incoming().flatten() {
        let handler = handler.clone();
        pool.execute(move || match Request::from_stream(&mut stream) {
            Ok(req) => handler(&req).write_to_stream(&mut stream),
            Err(err) => println!("Error: Invalid http request: {:?}", err),
        });
    }
}
