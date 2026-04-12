/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Vhost dispatch: admin router vs project proxy/static handler

use std::io::{self, Read, Write};
use std::net::TcpStream;

use const_format::formatcp;
use small_http::{Request, Response, Status};
use small_router::Router;

use crate::context::Context;
use crate::models::{BuildType, Project, ProjectStatus};

pub(crate) fn dispatch(
    req: &Request,
    ctx: &Context,
    admin_router: &Router<Context>,
    server_host: &str,
    deploy_path: &str,
) -> Response {
    let host = req
        .headers
        .get("Host")
        .or(req.headers.get("host"))
        .unwrap_or("")
        .to_string();

    // Strip port if present (e.g. localhost:8080)
    let host = host.split(':').next().unwrap_or("");

    if host == server_host || host.is_empty() {
        // Admin domain - route to admin router
        admin_router.handle(req)
    } else if let Some(subdomain) = host.strip_suffix(&format!(".{server_host}")) {
        // Project subdomain - find project and proxy/serve
        handle_project(req, ctx, subdomain, deploy_path)
    } else {
        // Unknown host - also route to admin router (handles localhost, etc.)
        admin_router.handle(req)
    }
}

fn handle_project(req: &Request, ctx: &Context, project_name: &str, deploy_path: &str) -> Response {
    let project = match ctx
        .database
        .query::<Project>(
            formatcp!(
                "SELECT {} FROM projects WHERE name = ? LIMIT 1",
                Project::columns()
            ),
            project_name.to_string(),
        )
        .ok()
        .and_then(|mut rows| rows.next())
        .and_then(|r| r.ok())
    {
        Some(p) => p,
        None => {
            return Response::new()
                .status(Status::NotFound)
                .body("404 Project not found");
        }
    };

    if project.status != ProjectStatus::Running {
        return Response::new()
            .status(Status::ServiceUnavailable)
            .body("503 Project not running");
    }

    match project.build_type {
        BuildType::Static => serve_static(req, &project, deploy_path),
        BuildType::Docker => {
            let is_websocket = req
                .headers
                .get("Upgrade")
                .map(|v| v.to_lowercase() == "websocket")
                .unwrap_or(false);
            if is_websocket {
                proxy_websocket(req, &project)
            } else {
                proxy_docker(req, &project)
            }
        }
        BuildType::Unknown => Response::new()
            .status(Status::ServiceUnavailable)
            .body("503 Project build type unknown"),
    }
}

fn serve_static(req: &Request, project: &Project, deploy_path: &str) -> Response {
    let base = if project.base_dir.is_empty() {
        format!("{deploy_path}/{}/repo", project.name)
    } else {
        format!("{deploy_path}/{}/repo/{}", project.name, project.base_dir)
    };

    let mut path = req.url.path().to_string();
    if path.ends_with('/') || path.is_empty() {
        path = format!("{path}index.html");
    }

    let file_path = format!("{base}{path}");

    // Security: ensure resolved path stays within base dir
    let canonical_base = match std::fs::canonicalize(&base) {
        Ok(p) => p,
        Err(_) => {
            return Response::new()
                .status(Status::ServiceUnavailable)
                .body("503 Project files not available");
        }
    };

    let file_path_obj = std::path::Path::new(&file_path);
    let canonical_file = match std::fs::canonicalize(file_path_obj) {
        Ok(p) => p,
        Err(_) => {
            // Try index.html fallback for SPA-style static sites
            let fallback = format!("{base}/index.html");
            match std::fs::read(&fallback) {
                Ok(data) => {
                    return Response::new()
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(data);
                }
                Err(_) => {
                    return Response::new()
                        .status(Status::NotFound)
                        .body("404 Not Found");
                }
            }
        }
    };

    if !canonical_file.starts_with(&canonical_base) {
        return Response::new()
            .status(Status::Forbidden)
            .body("403 Forbidden");
    }

    match std::fs::read(&canonical_file) {
        Ok(data) => {
            let mime = mime_guess::from_path(&canonical_file)
                .first_or_octet_stream()
                .to_string();
            Response::new().header("Content-Type", mime).body(data)
        }
        Err(_) => Response::new()
            .status(Status::NotFound)
            .body("404 Not Found"),
    }
}

fn proxy_websocket(req: &Request, project: &Project) -> Response {
    let container_ip = match &project.container_ip {
        Some(ip) if !ip.is_empty() => ip.clone(),
        _ => {
            return Response::new()
                .status(Status::ServiceUnavailable)
                .body("503 Project not deployed yet");
        }
    };
    let container_port = project.container_port.unwrap_or(3000);
    let addr = format!("{container_ip}:{container_port}");

    let mut upstream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(_) => {
            return Response::new()
                .status(Status::BadGateway)
                .body("502 Bad Gateway");
        }
    };

    // Build raw HTTP upgrade request — cannot use Request::fetch() because
    // write_to_stream forces Connection: close which kills the upgrade.
    let path_and_query = if req.url.query().is_none_or(|q| q.is_empty()) {
        req.url.path().to_string()
    } else {
        format!("{}?{}", req.url.path(), req.url.query().unwrap_or(""))
    };
    let original_host = req
        .headers
        .get("Host")
        .or(req.headers.get("host"))
        .unwrap_or("")
        .to_string();

    let mut raw = format!("{} {} HTTP/1.1\r\n", req.method, path_and_query);
    raw.push_str(&format!("Host: {addr}\r\n"));
    // Forward all client headers except Host and Content-Length (no body on upgrade)
    for (name, value) in &req.headers {
        let lower = name.to_lowercase();
        if lower != "host" && lower != "content-length" {
            raw.push_str(&format!("{name}: {value}\r\n"));
        }
    }
    raw.push_str(&format!("X-Forwarded-For: {}\r\n", req.ip()));
    raw.push_str(&format!("X-Forwarded-Host: {original_host}\r\n"));
    raw.push_str("X-Forwarded-Proto: http\r\n");
    raw.push_str("\r\n");

    if upstream.write_all(raw.as_bytes()).is_err() {
        return Response::new()
            .status(Status::BadGateway)
            .body("502 Bad Gateway");
    }

    // Read upstream response headers byte-by-byte until \r\n\r\n
    let mut header_buf = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];
    loop {
        match upstream.read_exact(&mut byte) {
            Ok(_) => {
                header_buf.push(byte[0]);
                if header_buf.ends_with(b"\r\n\r\n") {
                    break;
                }
                if header_buf.len() > 8192 {
                    return Response::new()
                        .status(Status::BadGateway)
                        .body("502 Bad Gateway");
                }
            }
            Err(_) => {
                return Response::new()
                    .status(Status::BadGateway)
                    .body("502 Bad Gateway");
            }
        }
    }

    // Verify upstream accepted the upgrade
    let header_str = String::from_utf8_lossy(&header_buf);
    let mut lines = header_str.lines();
    if !lines.next().unwrap_or("").contains("101") {
        return Response::new()
            .status(Status::BadGateway)
            .body("502 Upstream rejected WebSocket upgrade");
    }

    // Build 101 response, forwarding the upstream's handshake headers
    let mut res = Response::new().status(Status::SwitchingProtocols);
    for line in lines {
        if let Some((name, value)) = line.split_once(": ") {
            res = res.header(name, value.trim_end_matches('\r'));
        }
    }

    // Bidirectional pipe: client <-> upstream
    res.takeover(move |client| {
        let mut upstream_r = match upstream.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut client_w = match client.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut upstream_w = upstream;
        let mut client_r = client;

        // upstream -> client (background thread)
        std::thread::spawn(move || {
            let _ = io::copy(&mut upstream_r, &mut client_w);
            let _ = client_w.shutdown(std::net::Shutdown::Write);
        });

        // client -> upstream (this thread)
        let _ = io::copy(&mut client_r, &mut upstream_w);
        let _ = upstream_w.shutdown(std::net::Shutdown::Write);
    })
}

fn proxy_docker(req: &Request, project: &Project) -> Response {
    let container_ip = match &project.container_ip {
        Some(ip) if !ip.is_empty() => ip.clone(),
        _ => {
            return Response::new()
                .status(Status::ServiceUnavailable)
                .body("503 Project not deployed yet");
        }
    };
    let container_port = project.container_port.unwrap_or(3000);

    let path_and_query = if req.url.query().is_none_or(|q| q.is_empty()) {
        req.url.path().to_string()
    } else {
        format!("{}?{}", req.url.path(), req.url.query().unwrap_or(""))
    };
    let target_url = format!("http://{container_ip}:{container_port}{path_and_query}");

    // Hop-by-hop headers must not be forwarded in either direction (RFC 7230 §6.1).
    // Content-Length is also excluded — write_to_stream sets it from the actual body.
    const HOP_BY_HOP: &[&str] = &[
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
        "content-length",
    ];

    let original_host = req
        .headers
        .get("Host")
        .or(req.headers.get("host"))
        .unwrap_or("")
        .to_string();

    let mut proxy_req = Request::with_method_and_url(req.method, &target_url);
    for (name, value) in &req.headers {
        if !HOP_BY_HOP.contains(&name.to_lowercase().as_str()) && name.to_lowercase() != "host" {
            proxy_req = proxy_req.header(name, value);
        }
    }
    proxy_req = proxy_req
        .header("X-Forwarded-For", req.ip().to_string())
        .header("X-Forwarded-Host", &original_host)
        .header("X-Forwarded-Proto", "http");

    if let Some(body) = &req.body {
        proxy_req = proxy_req.body(body.clone());
    }

    match proxy_req.fetch() {
        Ok(upstream) => {
            // Strip hop-by-hop and Content-Length from upstream response.
            // Transfer-Encoding is critical: read_from_stream already decodes chunked
            // bodies, so forwarding the header would make browsers try to re-decode.
            // Content-Length is stripped so finish_headers sets it from the actual body.
            const STRIP_RESPONSE: &[&str] = &[
                "connection",
                "keep-alive",
                "proxy-authenticate",
                "proxy-authorization",
                "te",
                "trailer",
                "transfer-encoding",
                "upgrade",
                "content-length",
            ];
            let mut res = Response::new().status(upstream.status);
            for (name, value) in &upstream.headers {
                if !STRIP_RESPONSE.contains(&name.to_lowercase().as_str()) {
                    res = res.header(name, value);
                }
            }
            res.body(upstream.body)
        }
        Err(_) => Response::new()
            .status(Status::BadGateway)
            .body("502 Bad Gateway"),
    }
}
