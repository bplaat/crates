/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use std::env;
use std::net::{Ipv4Addr, TcpListener};
use std::process::Command;

use serde::Deserialize;
use small_http::{Request, Response, Status};

// MARK: Config
#[derive(Clone, Deserialize)]
struct Config {
    services: HashMap<String, Service>,
}

#[derive(Clone, Deserialize)]
struct Service {
    path: String,
    secret: String,
}

// MARK: Main
#[derive(Deserialize)]
struct WebhookBody {
    secret: String,
}

fn main() {
    // Read config
    let config_str = std::fs::read_to_string("config.yml").expect("Can't read config.yml");
    let config: Config = serde_yaml::from_str(&config_str).expect("Can't parse config.yml");

    let handler = move |req: &Request| -> Response {
        let path = req.url.path();
        println!("{} {}", req.method, path);

        // Get host
        let host = match req.headers.get("X-Forwarded-Host") {
            Some(host) => host,
            None => return Response::with_status(Status::BadRequest),
        };

        println!("Host: {}", host);

        // Get service
        let service = match config.services.get(host) {
            Some(service) => service,
            None => return Response::with_status(Status::NotFound),
        };

        // Get body
        let body =
            match serde_urlencoded::from_bytes::<WebhookBody>(req.body.as_deref().unwrap_or(&[])) {
                Ok(body) => body,
                Err(_) => return Response::with_status(Status::BadRequest),
            };

        // Check secret
        if body.secret != service.secret {
            println!("Secret wrong: {}", body.secret);
            return Response::with_status(Status::Forbidden);
        }

        // Run git pull in service path
        println!("Running git pull in: {}", service.path);
        Command::new("git")
            .arg("pull")
            .current_dir(&service.path)
            .output()
            .unwrap_or_else(|_| panic!("Failed to run git pull in: {}", service.path));
        Response::with_status(Status::Ok)
    };

    // Start server
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .unwrap_or_else(|_| panic!("Can't bind to port: {}", port));
    println!("Server is listening on: http://localhost:{}/", port);
    small_http::serve(listener, handler);
}
