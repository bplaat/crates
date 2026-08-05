/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use bwebview::{
    Event, EventLoopBuilder, FileDialog, LogicalSize, Theme, WebviewBuilder, WebviewEvent,
    WindowBuilder,
};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};

const FONT_SIZE: usize = 256 * 8;

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

// MARK: IPC messages
#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum IpcMessage {
    OpenFile {
        path: String,
    },
    OpenFileDialog,
    OpenFileDialogResponse {
        path: Option<String>,
    },
    OpenFont {
        path: String,
    },
    OpenFontResponse {
        ok: bool,
        data: Option<Vec<u8>>,
        error: Option<String>,
    },
    SaveFileDialog {
        filename: String,
    },
    SaveFileDialogResponse {
        path: Option<String>,
    },
    SaveFont {
        path: String,
        data: Vec<u8>,
    },
    SaveFontResponse {
        ok: bool,
        error: Option<String>,
    },
    ExportFileDialog {
        filename: String,
    },
    ExportFileDialogResponse {
        path: Option<String>,
    },
    ExportFile {
        path: String,
        text: String,
    },
    ExportFileResponse {
        ok: bool,
        error: Option<String>,
    },
}

// MARK: Main
fn main() {
    let startup_path = std::env::args().nth(1);
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "PixelFontEditor")
        .build();

    #[allow(unused_mut)]
    let mut window = WindowBuilder::new()
        .title("8x8 Pixel Font Editor")
        .size(LogicalSize::new(640.0, 860.0))
        .min_size(LogicalSize::new(640.0, 520.0))
        .background_color(if event_loop.theme() == Theme::Dark {
            0x222222
        } else {
            0xffffff
        })
        .center()
        .remember_window_state()
        .allow_file_drop(true)
        .build();

    let mut webview = WebviewBuilder::new(&window)
        .load_rust_embed::<WebAssets>()
        .build();

    if let Some(path) = startup_path {
        webview.add_user_script(
            format!(
                "window.startupFilePath = {};",
                serde_json::to_string(&path).expect("Failed to serialize startup path")
            ),
            bwebview::InjectionTime::DocumentStart,
        );
    }

    #[cfg(target_os = "macos")]
    let mut page_loaded = false;
    #[cfg(target_os = "macos")]
    let mut pending_open_path: Option<String> = None;
    event_loop.run(move |event| {
        #[cfg(target_os = "macos")]
        if let Event::Webview(WebviewEvent::PageLoadStart) = &event {
            page_loaded = false;
        }
        #[cfg(target_os = "macos")]
        if let Event::Webview(WebviewEvent::PageLoadFinish) = &event {
            page_loaded = true;
            if let Some(path) = pending_open_path.take() {
                webview.send_ipc_message(
                    serde_json::to_string(&IpcMessage::OpenFile { path })
                        .expect("Failed to serialize open file message"),
                );
            }
        }
        #[cfg(target_os = "macos")]
        if let Event::MacosOpenFiles(paths) = &event
            && let Some(path) = paths.first()
        {
            let path = path.to_string_lossy().into_owned();
            if page_loaded {
                webview.send_ipc_message(
                    serde_json::to_string(&IpcMessage::OpenFile { path })
                        .expect("Failed to serialize open file message"),
                );
            } else {
                pending_open_path = Some(path);
            }
        }
        if let Event::Webview(WebviewEvent::PageTitleChange(title)) = &event {
            window.set_title(title);
        }
        if let Event::Window(bwebview::WindowEvent::DroppedFile(path)) = &event {
            webview.send_ipc_message(
                serde_json::to_string(&IpcMessage::OpenFile {
                    path: path.to_string_lossy().into_owned(),
                })
                .expect("Failed to serialize open file message"),
            );
        }
        if let Event::Webview(WebviewEvent::MessageReceive(message)) = event {
            let Ok(ipc_message) = serde_json::from_str::<IpcMessage>(&message) else {
                return;
            };
            match ipc_message {
                IpcMessage::OpenFileDialog => {
                    let path = FileDialog::new()
                        .title("Open Pixel Font File")
                        .add_filter("Pixel Font files", &["pf"])
                        .pick_file()
                        .map(|p| p.to_string_lossy().into_owned());
                    let response = IpcMessage::OpenFileDialogResponse { path };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                IpcMessage::OpenFont { path } => {
                    let response = match std::fs::read(&path) {
                        Ok(bytes) if bytes.len() >= FONT_SIZE => IpcMessage::OpenFontResponse {
                            ok: true,
                            data: Some(bytes[..FONT_SIZE].to_vec()),
                            error: None,
                        },
                        Ok(_) => IpcMessage::OpenFontResponse {
                            ok: false,
                            data: None,
                            error: Some("File is too small (need at least 2048 bytes)".to_string()),
                        },
                        Err(e) => IpcMessage::OpenFontResponse {
                            ok: false,
                            data: None,
                            error: Some(e.to_string()),
                        },
                    };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                IpcMessage::SaveFileDialog { filename } => {
                    let path = FileDialog::new()
                        .title("Save Pixel Font File")
                        .set_file_name(&filename)
                        .add_filter("Pixel Font files", &["pf"])
                        .save_file()
                        .map(|p| p.to_string_lossy().into_owned());
                    let response = IpcMessage::SaveFileDialogResponse { path };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                IpcMessage::SaveFont { path, data } => {
                    let response = if data.len() != FONT_SIZE {
                        IpcMessage::SaveFontResponse {
                            ok: false,
                            error: Some(format!(
                                "Invalid font data size: expected {FONT_SIZE} bytes, got {}",
                                data.len()
                            )),
                        }
                    } else {
                        match std::fs::write(&path, &data) {
                            Ok(()) => IpcMessage::SaveFontResponse {
                                ok: true,
                                error: None,
                            },
                            Err(e) => IpcMessage::SaveFontResponse {
                                ok: false,
                                error: Some(e.to_string()),
                            },
                        }
                    };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                IpcMessage::ExportFileDialog { filename } => {
                    let path = FileDialog::new()
                        .title("Export Font File")
                        .set_file_name(&filename)
                        .save_file()
                        .map(|p| p.to_string_lossy().into_owned());
                    let response = IpcMessage::ExportFileDialogResponse { path };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                IpcMessage::ExportFile { path, text } => {
                    let response = match std::fs::write(&path, text.as_bytes()) {
                        Ok(()) => IpcMessage::ExportFileResponse {
                            ok: true,
                            error: None,
                        },
                        Err(e) => IpcMessage::ExportFileResponse {
                            ok: false,
                            error: Some(e.to_string()),
                        },
                    };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                _ => {}
            }
        }
    });
}
