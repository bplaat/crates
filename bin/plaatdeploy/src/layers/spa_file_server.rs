/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use rust_embed::Embed;
use small_http::{Method, Request, Response};

use crate::Context;

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

const CSP_VALUE: &str = concat!(
    "default-src 'self'; ",
    "script-src 'self'; ",
    "script-src-attr 'none'; ",
    "style-src 'self'; ",
    "style-src-attr 'none'; ",
    "img-src 'self' data:; ",
    "connect-src 'self'; ",
    "object-src 'none'; ",
    "frame-src 'none'; ",
    "frame-ancestors 'none'; ",
    "base-uri 'self'; ",
    "form-action 'self'; ",
    "worker-src 'none'",
);

pub(crate) fn spa_file_server_pre_layer(
    req: &Request,
    _: &mut Context,
) -> Result<Option<Response>> {
    let path = req.url.path();
    if path == "/api" || path.starts_with("/api/") {
        return Ok(None);
    }
    if !matches!(req.method, Method::Get | Method::Head) {
        return Ok(None);
    }
    if path == "/swagger-ui" {
        return Ok(Some(Response::with_redirect("/swagger-ui/index.html")));
    }

    let mut path = path.to_string();
    if path.ends_with('/') {
        path = format!("{path}index.html");
    }
    if let Some(file) = WebAssets::get(path.trim_start_matches('/')) {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        Ok(Some(
            Response::with_header("Content-Type", mime.to_string())
                .header("Content-Security-Policy", CSP_VALUE)
                .body(file.data),
        ))
    } else {
        let file = WebAssets::get("index.html")
            .ok_or_else(|| anyhow::anyhow!("index.html not found in embedded assets"))?;
        Ok(Some(
            Response::with_header("Content-Type", "text/html")
                .header("Content-Security-Policy", CSP_VALUE)
                .body(file.data),
        ))
    }
}

#[cfg(test)]
mod tests {
    use small_http::Status;

    use super::*;

    #[test]
    fn leaves_api_requests_to_the_router() {
        let mut ctx = Context::with_test_database().expect("create test database");
        let response =
            spa_file_server_pre_layer(&Request::get("http://localhost/api/projects"), &mut ctx)
                .expect("serve request");

        assert!(response.is_none());

        let response =
            spa_file_server_pre_layer(&Request::get("http://localhost/api-looking-page"), &mut ctx)
                .expect("serve request");
        assert!(response.is_some());

        let response =
            spa_file_server_pre_layer(&Request::post("http://localhost/settings"), &mut ctx)
                .expect("serve request");
        assert!(response.is_none());
    }

    #[test]
    fn serves_embedded_files_and_spa_fallback() {
        let mut ctx = Context::with_test_database().expect("create test database");
        let robots =
            spa_file_server_pre_layer(&Request::get("http://localhost/robots.txt"), &mut ctx)
                .expect("serve robots")
                .expect("robots response");
        let fallback =
            spa_file_server_pre_layer(&Request::get("http://localhost/projects/example"), &mut ctx)
                .expect("serve fallback")
                .expect("fallback response");

        assert_eq!(robots.status, Status::Ok);
        assert_eq!(robots.headers.get("Content-Type"), Some("text/plain"));
        assert_eq!(fallback.status, Status::Ok);
        assert_eq!(fallback.headers.get("Content-Type"), Some("text/html"));
        assert_eq!(
            fallback.headers.get("Content-Security-Policy"),
            Some(CSP_VALUE)
        );
    }

    #[test]
    fn redirects_swagger_ui_root() {
        let mut ctx = Context::with_test_database().expect("create test database");
        let response =
            spa_file_server_pre_layer(&Request::get("http://localhost/swagger-ui"), &mut ctx)
                .expect("serve redirect")
                .expect("redirect response");

        assert_eq!(response.status, Status::TemporaryRedirect);
        assert_eq!(
            response.headers.get("Location"),
            Some("/swagger-ui/index.html")
        );
    }
}
