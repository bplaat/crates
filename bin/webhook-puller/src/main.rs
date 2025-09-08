/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::net::{Ipv4Addr, TcpListener};
use std::process::Command;
use std::time::Duration;
use std::{env, thread};

use serde::Deserialize;
use small_http::{Method, Request, Response, Status};

// MARK: Config
type Config = HashMap<String, Service>;

#[derive(Clone, Deserialize)]
#[allow(dead_code)]
struct Service {
    path: String,
    secret: String,
    #[serde(default)]
    cloudflare_purge_everything: bool,
    #[serde(default)]
    cloudflare_api_token: String,
    #[serde(default)]
    cloudflare_zone_id: String,
}

fn main() {
    // Read config
    let config_str = std::fs::read_to_string("config.toml").expect("Can't read config.toml");
    let config: Config = basic_toml::from_str(&config_str).expect("Can't parse config.toml");

    // Server handler
    let handler = move |req: &Request| -> Response {
        let path = req.url.path();
        println!("{} {}", req.method, path);

        // Check if method is POST
        if req.method != Method::Post {
            return Response::with_status(Status::MethodNotAllowed);
        }

        // Get host
        let host = match req.headers.get("X-Forwarded-Host") {
            Some(host) => host,
            None => req.url.host().unwrap_or("localhost"),
        };
        println!("Host: {host}");

        // Get service
        let service = match config.get(host) {
            Some(service) => service,
            None => return Response::with_status(Status::NotFound),
        };

        // FIXME: Validate secret

        // Get X-GitHub-Event
        let event = req.headers.get("X-GitHub-Event").unwrap_or("push");
        if event != "push" {
            println!("Ignoring event: {event}");
            return Response::with_status(Status::Ok);
        }
        println!("GitHub Webhook Event: {event}");

        // Spawn git task thread
        let service_path = service.path.clone();
        thread::spawn(move || {
            // Sleep for 10 seconds
            thread::sleep(Duration::from_secs(10));

            // Run git commands
            println!("Running `git fetch origin` in: {service_path}");
            Command::new("git")
                .arg("fetch")
                .arg("origin")
                .current_dir(&service_path)
                .output()
                .unwrap_or_else(|_| panic!("Failed to run git pull in: {service_path}"));

            println!("Running `git reset --hard origin/master` in: {service_path}");
            Command::new("git")
                .arg("reset")
                .arg("--hard")
                .arg("origin/master")
                .current_dir(&service_path)
                .output()
                .unwrap_or_else(|_| panic!("Failed to run git reset in: {service_path}"));
        });

        // Spawn Cloudflare purge thread
        if service.cloudflare_purge_everything {
            let api_token = service.cloudflare_api_token.clone();
            let zone_id = service.cloudflare_zone_id.clone();
            thread::spawn(move || {
                println!("Purging Cloudflare cache...");
                let output = Command::new("curl")
                    .arg("-X")
                    .arg("POST")
                    .arg(format!(
                        "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
                        zone_id
                    ))
                    .arg("-H")
                    .arg(format!("Authorization: Bearer {}", api_token))
                    .arg("-H")
                    .arg("Content-Type: application/json")
                    .arg("-d")
                    .arg(r#"{"purge_everything":true}"#)
                    .output()
                    .unwrap_or_else(|_| panic!("Failed to purge Cloudflare cache"));
                if output.status.code() == Some(0) {
                    println!(
                        "Cloudflare cache purged: {}",
                        String::from_utf8_lossy(&output.stdout)
                    );
                } else {
                    println!(
                        "Failed to purge Cloudflare cache: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            });
        }

        Response::with_status(Status::Ok)
    };

    // Start server
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .unwrap_or_else(|_| panic!("Can't bind to port: {port}"));
    println!("Server is listening on: http://localhost:{port}/");
    small_http::serve(listener, handler);
}
