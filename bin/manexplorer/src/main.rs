/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::process::Command;

use bwebview::{Event, EventLoopBuilder, LogicalSize, WebviewBuilder};
use rust_embed::Embed;
use serde::Serialize;
use small_http::{Request, Response, Status};
use small_router::RouterBuilder;

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

#[derive(Serialize)]
struct ManPage {
    page: i32,
    names: Vec<String>,
}

fn man_index(_req: &Request, _ctx: &()) -> Response {
    // List all directories in /usr/share/man
    let mut pages = Vec::new();
    if let Ok(dir_iter) = std::fs::read_dir("/usr/share/man") {
        for dir in dir_iter.flatten() {
            let path = dir.path();
            if path.is_dir()
                && let Some(page_str) = path.file_name().and_then(|n| n.to_str())
            {
                // Try to parse the page number from "man1", "man2", etc.
                if let Some(page_num) = page_str
                    .strip_prefix("man")
                    .and_then(|n| n.parse::<i32>().ok())
                {
                    // List all files in this manX directory
                    let mut names = Vec::new();
                    if let Ok(file_iter) = std::fs::read_dir(&path) {
                        for file in file_iter.flatten() {
                            if let Some(file_name) = file.file_name().to_str() {
                                // Remove file extension if present (e.g., "ls.1.gz" -> "ls")
                                let name =
                                    file_name.split('.').next().unwrap_or(file_name).to_string();
                                if !name.is_empty() && !names.contains(&name) {
                                    names.push(name);
                                }
                            }
                        }
                    }
                    names.sort_by(|a, b| {
                        a.to_lowercase()
                            .cmp(&b.to_lowercase())
                            .then_with(|| a.cmp(b))
                    });
                    pages.push(ManPage {
                        page: page_num,
                        names,
                    });
                }
            }
        }
    }
    pages.sort_by_key(|entry| entry.page);
    Response::with_json(&pages)
}

fn man_show(req: &Request, _ctx: &()) -> Response {
    let page = req
        .params
        .get("page")
        .expect("page param should be present");
    let name = req
        .params
        .get("name")
        .expect("name param should be present");
    let output = Command::new("man")
        .arg("-P")
        .arg("col -b")
        .arg(page)
        .arg(name)
        .output()
        .expect("Failed to execute man command");
    Response::with_body(String::from_utf8_lossy(&output.stdout).to_string())
}

fn main() {
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "ManExplorer")
        .build();

    let router = RouterBuilder::new()
        .get("/api/man", man_index)
        .get("/api/man/:page/:name", man_show)
        .build();

    #[allow(unused_mut)]
    let mut webview_builder = WebviewBuilder::new()
        .title("Man Explorer")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(800.0, 480.0))
        .center()
        .remember_window_state()
        .load_rust_embed_with_custom_handler::<WebAssets>(move |req| {
            let res = router.handle(req);
            if res.status != Status::NotFound {
                Some(res)
            } else {
                None
            }
        });
    #[cfg(target_os = "macos")]
    {
        webview_builder =
            webview_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Hidden);
    }
    let mut webview = webview_builder.build();

    #[cfg(target_os = "macos")]
    webview.add_user_script(
        format!(
            "document.documentElement.style.setProperty('--macos-titlebar-height', '{}px');",
            webview.macos_titlebar_size().height
        ),
        bwebview::InjectionTime::DocumentStart,
    );

    event_loop.run(move |event| match event {
        Event::PageTitleChanged(title) => webview.set_title(title),
        #[cfg(target_os = "macos")]
        Event::MacosWindowFullscreenChanged(is_fullscreen) => {
            if is_fullscreen {
                webview.evaluate_script("document.body.classList.add('is-fullscreen');");
            } else {
                webview.evaluate_script("document.body.classList.remove('is-fullscreen');");
            }
        }
        _ => {}
    });
}
