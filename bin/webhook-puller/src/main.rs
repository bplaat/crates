/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::net::{Ipv4Addr, TcpListener};
use std::process::Command;
use std::time::Duration;
use std::{env, thread};

use ini::ConfigFile;
use small_http::{Method, Request, Response, Status};

fn main() {
    // Read config
    let config = ConfigFile::load_from_path("config.ini").expect("Can't read config.ini");

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

        // Check if service exists
        if !config.groups().any(|group| group == host) {
            return Response::with_status(Status::NotFound);
        }

        // FIXME: Validate secret

        // Get X-GitHub-Event
        let event = req.headers.get("X-GitHub-Event").unwrap_or("push");
        if event != "push" {
            println!("Ignoring event: {event}");
            return Response::with_status(Status::Ok);
        }
        println!("GitHub Webhook Event: {event}");

        // Spawn git task thread
        let service_path = config
            .read_string(host, "path")
            .unwrap_or_else(|| panic!("No path configured for host: {host}"))
            .to_string();
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
        if config
            .read_bool(host, "cloudflare_purge_everything")
            .unwrap_or(false)
        {
            let api_token = config
                .read_string(host, "cloudflare_api_token")
                .unwrap_or_else(|| panic!("No Cloudflare API token configured for host: {host}"))
                .to_string();
            let zone_id = config
                .read_string(host, "cloudflare_zone_id")
                .unwrap_or_else(|| panic!("No Cloudflare zone ID configured for host: {host}"))
                .to_string();
            thread::spawn(move || {
                println!("Purging Cloudflare cache...");
                let output = Command::new("curl")
                    .arg("-X")
                    .arg("POST")
                    .arg(format!(
                        "https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache"
                    ))
                    .arg("-H")
                    .arg(format!("Authorization: Bearer {api_token}"))
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
