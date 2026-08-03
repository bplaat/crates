/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod catalog;
mod engine;

use std::collections::HashSet;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;

use bwebview::{
    Event, EventLoopBuilder, EventLoopProxy, LogicalSize, ProgressBarState, Theme, WebviewBuilder,
    WebviewEvent, WindowBuilder,
};
use catalog::Catalog;
use engine::CleanupResult;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum IpcRequest {
    Initialize,
    StartScan,
    CancelScan,
    StartClean { cleanup_ids: Vec<String> },
}

#[derive(Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum IpcPush<'a> {
    Catalog {
        catalog: &'a Catalog,
        disk_free: Option<u64>,
        is_administrator: bool,
    },
    OperationStarted {
        operation: &'a str,
        total: usize,
    },
    CleanupProgress {
        operation: &'a str,
        cleanup_id: &'a str,
        index: usize,
        total: usize,
    },
    ScanResult {
        result: CleanupResult,
    },
    ScanFinished {
        cancelled: bool,
    },
    CleanResult {
        result: CleanupResult,
    },
    CleanFinished {
        recovered_bytes: u64,
    },
    DiskFreeUpdated {
        disk_free: Option<u64>,
    },
    FatalError {
        message: String,
    },
}

enum WorkerCommand {
    Initialize,
    Scan,
    Clean(Vec<String>),
}

fn send_push(proxy: &Arc<EventLoopProxy>, push: IpcPush<'_>) {
    match serde_json::to_string(&push) {
        Ok(json) => proxy.send_user_event(json),
        Err(error) => {
            let fallback = serde_json::to_string(&IpcPush::FatalError {
                message: error.to_string(),
            })
            .expect("fatal error should serialize");
            proxy.send_user_event(fallback);
        }
    }
}

fn worker(
    receiver: mpsc::Receiver<WorkerCommand>,
    proxy: Arc<EventLoopProxy>,
    cancelled: Arc<AtomicBool>,
) {
    let is_administrator = is_process_elevated();
    let catalog = match Catalog::load() {
        Ok(catalog) => catalog,
        Err(error) => {
            send_push(
                &proxy,
                IpcPush::FatalError {
                    message: error.to_string(),
                },
            );
            return;
        }
    };

    for command in receiver {
        match command {
            WorkerCommand::Initialize => send_push(
                &proxy,
                IpcPush::Catalog {
                    catalog: &catalog,
                    disk_free: engine::disk_free_space(),
                    is_administrator,
                },
            ),
            WorkerCommand::Scan => {
                cancelled.store(false, Ordering::Relaxed);
                let rules: Vec<_> = catalog.rules().collect();
                proxy.set_progress_bar(ProgressBarState::Indeterminate);
                send_push(
                    &proxy,
                    IpcPush::OperationStarted {
                        operation: "scan",
                        total: rules.len(),
                    },
                );
                for (index, cleanup) in rules.iter().enumerate() {
                    if cancelled.load(Ordering::Relaxed) {
                        break;
                    }
                    send_push(
                        &proxy,
                        IpcPush::CleanupProgress {
                            operation: "scan",
                            cleanup_id: &cleanup.id,
                            index,
                            total: rules.len(),
                        },
                    );
                    send_push(
                        &proxy,
                        IpcPush::ScanResult {
                            result: engine::scan(cleanup, &cancelled, is_administrator),
                        },
                    );
                    proxy.set_progress_bar(ProgressBarState::Normal(
                        (index + 1) as f64 / rules.len() as f64,
                    ));
                }
                send_push(
                    &proxy,
                    IpcPush::ScanFinished {
                        cancelled: cancelled.load(Ordering::Relaxed),
                    },
                );
                proxy.set_progress_bar(ProgressBarState::None);
            }
            WorkerCommand::Clean(ids) => {
                let ids: HashSet<_> = ids.into_iter().collect();
                let rules: Vec<_> = catalog
                    .rules()
                    .filter(|cleanup| {
                        ids.contains(&cleanup.id)
                            && (is_administrator || !cleanup.requires_administrator)
                    })
                    .collect();
                proxy.set_progress_bar(ProgressBarState::Indeterminate);
                send_push(
                    &proxy,
                    IpcPush::OperationStarted {
                        operation: "clean",
                        total: rules.len(),
                    },
                );
                let mut had_errors = false;
                let mut recovered_bytes = 0;
                for (index, cleanup) in rules.iter().enumerate() {
                    send_push(
                        &proxy,
                        IpcPush::CleanupProgress {
                            operation: "clean",
                            cleanup_id: &cleanup.id,
                            index,
                            total: rules.len(),
                        },
                    );
                    let result = engine::clean(cleanup, is_administrator);
                    had_errors |= !result.errors.is_empty();
                    recovered_bytes += result.cleaned_bytes;
                    send_push(&proxy, IpcPush::CleanResult { result });
                    proxy.set_progress_bar(ProgressBarState::Normal(
                        (index + 1) as f64 / rules.len() as f64,
                    ));
                }
                send_push(&proxy, IpcPush::CleanFinished { recovered_bytes });
                proxy.set_progress_bar(if had_errors {
                    ProgressBarState::Error(1.0)
                } else {
                    ProgressBarState::None
                });
                send_push(
                    &proxy,
                    IpcPush::DiskFreeUpdated {
                        disk_free: engine::disk_free_space(),
                    },
                );
            }
        }
    }
}

#[cfg(windows)]
fn is_process_elevated() -> bool {
    // High and System integrity levels identify an elevated token. Unlike checking
    // administrator group membership, this remains correct for UAC-filtered tokens.
    let Some(executable) = engine::system_executable("whoami.exe") else {
        return false;
    };
    std::process::Command::new(executable)
        .arg("/groups")
        .output()
        .is_ok_and(|output| {
            let groups = String::from_utf8_lossy(&output.stdout);
            groups.contains("S-1-16-12288") || groups.contains("S-1-16-16384")
        })
}

#[cfg(not(windows))]
fn is_process_elevated() -> bool {
    false
}

fn main() {
    if !cfg!(target_os = "windows") {
        eprintln!("Binman can only be run on Windows");
        exit(1);
    }

    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "Binman")
        .build();
    let proxy = Arc::new(event_loop.create_proxy());
    let cancelled = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel();
    let worker_proxy = Arc::clone(&proxy);
    let worker_cancelled = Arc::clone(&cancelled);
    thread::spawn(move || worker(receiver, worker_proxy, worker_cancelled));

    let mut window = WindowBuilder::new()
        .title("Binman")
        .size(LogicalSize::new(1080.0, 720.0))
        .min_size(LogicalSize::new(820.0, 560.0))
        .background_color(if event_loop.theme() == Theme::Dark {
            0x17191d
        } else {
            0xf7f8fa
        })
        .center()
        .build();

    let mut webview = WebviewBuilder::new(&window)
        .load_rust_embed::<WebAssets>()
        .build();

    event_loop.run(move |event| match event {
        Event::UserEvent(json) => webview.send_ipc_message(json),
        Event::Webview(WebviewEvent::PageTitleChange(title)) => window.set_title(title),
        Event::Webview(WebviewEvent::MessageReceive(message)) => {
            match serde_json::from_str::<IpcRequest>(&message) {
                Ok(request) => match request {
                    IpcRequest::Initialize => {
                        _ = sender.send(WorkerCommand::Initialize);
                    }
                    IpcRequest::StartScan => {
                        _ = sender.send(WorkerCommand::Scan);
                    }
                    IpcRequest::CancelScan => {
                        cancelled.store(true, Ordering::Relaxed);
                    }
                    IpcRequest::StartClean { cleanup_ids } => {
                        _ = sender.send(WorkerCommand::Clean(cleanup_ids));
                    }
                },
                Err(error) => {
                    let push = IpcPush::FatalError {
                        message: format!("Invalid application request: {error}"),
                    };
                    if let Ok(json) = serde_json::to_string(&push) {
                        webview.send_ipc_message(json);
                    }
                }
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_request_accepts_browser_field_names() {
        let request = serde_json::from_str::<IpcRequest>(
            r#"{"type":"startClean","cleanupIds":["windows-user-temp"]}"#,
        )
        .expect("browser cleanup request should deserialize");

        match request {
            IpcRequest::StartClean { cleanup_ids } => {
                assert_eq!(cleanup_ids, ["windows-user-temp"]);
            }
            _ => panic!("expected cleanup request"),
        }
    }

    #[test]
    fn progress_push_uses_browser_field_names() {
        let json = serde_json::to_string(&IpcPush::CleanupProgress {
            operation: "scan",
            cleanup_id: "windows-user-temp",
            index: 0,
            total: 1,
        })
        .expect("progress push should serialize");

        assert!(json.contains(r#""type":"cleanupProgress""#));
        assert!(json.contains(r#""cleanupId":"windows-user-temp""#));
        assert!(!json.contains("cleanup_id"));
    }
}
