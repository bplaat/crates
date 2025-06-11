/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use std::net::{Ipv4Addr, TcpListener};
use std::process::Command;
use std::time::Duration;
use std::{env, thread};

use serde::Deserialize;
use small_http::{Method, Request, Response, Status};

// MARK: Config
#[derive(Clone, Deserialize)]
struct Config {
    services: HashMap<String, Service>,
}

#[derive(Clone, Deserialize)]
#[allow(dead_code)]
struct Service {
    path: String,
    secret: String,
}

fn main() {
    // Read config
    let config_str = std::fs::read_to_string("config.yml").expect("Can't read config.yml");
    let config: Config = serde_yaml::from_str(&config_str).expect("Can't parse config.yml");

    let handler = move |req: &Request| -> Response {
        let path = req.url.path();
        println!("{} {}", req.method, path);

        // Check if method is POST
        if req.method != Method::Post {
            return Response::with_status(Status::MethodNotAllowed);
        }

        // Get host
        let host = match req.headers.get("X-Forwarded-Host") {
            Some(host) => host.as_str(),
            None => req.url.host().unwrap_or("localhost"),
        };
        println!("Host: {}", host);

        // Get service
        let service = match config.services.get(host) {
            Some(service) => service,
            None => return Response::with_status(Status::NotFound),
        };

        // FIXME: Validate secret

        // Get X-GitHub-Event
        let event = match req.headers.get("X-GitHub-Event") {
            Some(event) => event,
            None => "push",
        };
        if event != "push" {
            println!("Ignoring event: {}", event);
            return Response::with_status(Status::Ok);
        }
        println!("Event: {}", event);

        // Spawn git task thread
        let service_path = service.path.clone();
        thread::spawn(move || {
            // Sleep for 0 seconds
            thread::sleep(Duration::from_secs(60));

            // Run git commands
            println!("Running `git fetch` origin in: {}", service_path);
            Command::new("git")
                .arg("fetch")
                .arg("origin")
                .current_dir(&service_path)
                .output()
                .unwrap_or_else(|_| panic!("Failed to run git pull in: {}", service_path));

            println!(
                "Running `git reset --hard origin/master` in: {}",
                service_path
            );
            Command::new("git")
                .arg("reset")
                .arg("--hard")
                .arg("origin/master")
                .current_dir(&service_path)
                .output()
                .unwrap_or_else(|_| panic!("Failed to run git reset in: {}", service_path));
        });

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
