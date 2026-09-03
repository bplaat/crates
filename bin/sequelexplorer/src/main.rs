/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod database;
mod ipc;
mod schema;
mod sql_transfer;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use bsql::{Connection, PoolOptions, SqliteMode};
use bwebview::{
    Event, EventLoopBuilder, FileDialog, LogicalSize, Theme, WebviewBuilder, WebviewEvent,
    WindowBuilder,
};
use rust_embed::Embed;
use small_http::Status;
use small_router::RouterBuilder;
use zeroize::Zeroizing;

use crate::database::{
    DatabaseState, MysqlConnectionPendingGuard, OpenMysqlRequest, State, db_databases, db_query,
    db_raw_query, db_table_data, db_table_delete, db_table_insert, db_table_update, db_tables,
    db_users, db_users_create, db_users_delete, db_users_update, open_mysql,
    replace_database_state_if_current,
};
use crate::ipc::IpcMessage;
use crate::schema::{db_table_schema, db_table_schema_update};
use crate::sql_transfer::{export_sql, import_sql};

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

fn main() {
    let startup_path = std::env::args().nth(1);
    let state: State = Arc::new(Mutex::new(DatabaseState::default()));
    #[allow(unused_mut)]
    let mut event_loop_builder = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "SequelExplorer")
        .single_instance(false);
    #[cfg(target_os = "macos")]
    {
        use bwebview::{Accelerator, KeyCode, MenuBarBuilder, MenuBuilder, MenuItem, Modifiers};

        event_loop_builder = event_loop_builder.macos_set_menu(
            MenuBarBuilder::new()
                .menu(
                    MenuBuilder::new("File")
                        .item(
                            MenuItem::new("Connect to Database...", "open")
                                .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::KeyO)),
                        )
                        .separator()
                        .item(MenuItem::new("Import SQL...", "importSql"))
                        .item(MenuItem::new("Export SQL...", "exportSql"))
                        .separator(),
                )
                .menu(
                    MenuBuilder::new("View")
                        .item(
                            MenuItem::new("Data", "showData")
                                .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::Digit1)),
                        )
                        .item(
                            MenuItem::new("Schema", "showSchema")
                                .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::Digit2)),
                        )
                        .item(
                            MenuItem::new("Query", "showQuery")
                                .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::Digit3)),
                        ),
                )
                .menu(
                    MenuBuilder::new("Query")
                        .item(
                            MenuItem::new("Run Query", "runQuery")
                                .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::Enter)),
                        )
                        .item(
                            MenuItem::new("Clear Query", "clearQuery")
                                .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::KeyK)),
                        ),
                ),
        );
    }
    let event_loop = event_loop_builder.build();

    let router = RouterBuilder::<State>::with(Arc::clone(&state))
        .get("/api/databases", db_databases)
        .get("/api/users", db_users)
        .post("/api/users", db_users_create)
        .put("/api/users", db_users_update)
        .delete("/api/users", db_users_delete)
        .get("/api/tables", db_tables)
        .get("/api/table/:name/data", db_table_data)
        .post("/api/table/:name/data", db_table_insert)
        .put("/api/table/:name/data", db_table_update)
        .delete("/api/table/:name/data", db_table_delete)
        .get("/api/table/:name/schema", db_table_schema)
        .put("/api/table/:name/schema", db_table_schema_update)
        .post("/api/query", db_query)
        .post("/api/query/raw", db_raw_query)
        .build();
    let event_loop_proxy = Arc::new(event_loop.create_proxy());
    let mysql_connection_pending = Arc::new(AtomicBool::new(false));
    let connection_generation = Arc::new(AtomicU64::new(0));

    #[allow(unused_mut)]
    let mut window_builder = WindowBuilder::new()
        .title("Sequel Explorer")
        .size(LogicalSize::new(1200.0, 768.0))
        .min_size(LogicalSize::new(800.0, 480.0))
        .background_color(if event_loop.theme() == Theme::Dark {
            0x222222
        } else {
            0xffffff
        })
        .center()
        .remember_window_state()
        .allow_file_drop(true);
    #[cfg(target_os = "macos")]
    {
        window_builder = window_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Hidden);
    }
    let mut window = window_builder.build();

    let mut webview = WebviewBuilder::new(&window)
        .load_rust_embed_with_custom_handler::<WebAssets>(move |req| {
            let res = router.handle(req);
            if res.status != Status::NotFound {
                Some(res)
            } else {
                None
            }
        })
        .build();

    #[cfg(target_os = "macos")]
    webview.add_user_script(
        format!(
            "document.documentElement.style.setProperty('--macos-titlebar-height', '{}px');",
            window.macos_titlebar_size().height
        ),
        bwebview::InjectionTime::DocumentStart,
    );

    #[cfg(target_os = "macos")]
    let mut page_ready = false;
    #[cfg(target_os = "macos")]
    let mut pending_menu_action: Option<String> = None;
    let mut pending_open_path = startup_path;
    let mut initial_restore_sent = false;
    event_loop.run(move |event| match event {
        #[cfg(target_os = "macos")]
        Event::Webview(WebviewEvent::PageLoadStart) => page_ready = false,
        #[cfg(target_os = "macos")]
        Event::MacosOpenFiles(paths) => {
            if let Some(path) = paths.first() {
                let path = path.to_string_lossy().into_owned();
                if page_ready {
                    webview.send_ipc_message(
                        serde_json::to_string(&IpcMessage::OpenFile { path })
                            .expect("Failed to serialize open file message"),
                    );
                } else {
                    pending_open_path = Some(path);
                }
            }
        }
        Event::Webview(WebviewEvent::PageTitleChange(title)) => window.set_title(title),
        #[cfg(target_os = "macos")]
        Event::MacosMenuItem(action) => {
            if page_ready {
                webview.send_ipc_message(
                    serde_json::to_string(&IpcMessage::MenuAction { action })
                        .expect("Failed to serialize menu action"),
                );
            } else {
                pending_menu_action = Some(action);
            }
        }
        Event::Window(bwebview::WindowEvent::DroppedFile(path)) => {
            webview.send_ipc_message(
                serde_json::to_string(&IpcMessage::OpenFile {
                    path: path.to_string_lossy().into_owned(),
                })
                .expect("Failed to serialize open file message"),
            );
        }
        #[cfg(target_os = "macos")]
        Event::Window(bwebview::WindowEvent::MacosFullscreenChange(is_fullscreen)) => {
            if is_fullscreen {
                webview.evaluate_script("document.body.classList.add('is-fullscreen');");
            } else {
                webview.evaluate_script("document.body.classList.remove('is-fullscreen');");
            }
        }
        Event::Webview(WebviewEvent::MessageReceive(message)) => {
            let message = Zeroizing::new(message);
            let message = match serde_json::from_str(&message) {
                Ok(message) => message,
                Err(error) => {
                    eprintln!("Ignoring invalid IPC message: {error}");
                    return;
                }
            };
            match message {
                IpcMessage::Ready => {
                    #[cfg(target_os = "macos")]
                    {
                        page_ready = true;
                    }
                    if !initial_restore_sent {
                        initial_restore_sent = true;
                        if let Some(path) = pending_open_path.take() {
                            webview.send_ipc_message(
                                serde_json::to_string(&IpcMessage::OpenFile { path })
                                    .expect("Failed to serialize open file message"),
                            );
                        } else {
                            webview.send_ipc_message(
                                serde_json::to_string(&IpcMessage::RestoreLastFile)
                                    .expect("Failed to serialize restore last file message"),
                            );
                        }
                    }
                    #[cfg(target_os = "macos")]
                    if let Some(action) = pending_menu_action.take() {
                        webview.send_ipc_message(
                            serde_json::to_string(&IpcMessage::MenuAction { action })
                                .expect("Failed to serialize menu action"),
                        );
                    }
                }
                IpcMessage::RestoreLastFile => {}
                IpcMessage::OpenFileDialog { request_id } => {
                    let path = FileDialog::new()
                        .parent(&window)
                        .title("Open SQLite Database")
                        .add_filter("SQLite databases", &["db", "sqlite", "sqlite3"])
                        .pick_file()
                        .map(|p| p.to_string_lossy().into_owned());
                    let response = IpcMessage::OpenFileDialogResponse { request_id, path };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                IpcMessage::OpenDatabase { request_id, path } => {
                    let request_generation =
                        connection_generation.fetch_add(1, Ordering::AcqRel) + 1;
                    let result = Connection::open_sqlite(
                        &path,
                        SqliteMode::ReadWrite,
                        PoolOptions::single_connection(),
                    );
                    let (ok, error) = match result {
                        Ok(conn) => {
                            if replace_database_state_if_current(
                                &state,
                                &connection_generation,
                                request_generation,
                                DatabaseState::sqlite(conn),
                            ) {
                                (true, None)
                            } else {
                                (false, Some("Connection request was superseded".to_string()))
                            }
                        }
                        Err(e) => (false, Some(e.to_string())),
                    };
                    let response = IpcMessage::OpenDatabaseResponse {
                        request_id,
                        ok,
                        error,
                    };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                IpcMessage::ImportSql { request_id } => {
                    let path = FileDialog::new()
                        .parent(&window)
                        .title("Import SQL")
                        .add_filter("SQL files", &["sql"])
                        .pick_file();
                    if let Some(path) = path {
                        let state = Arc::clone(&state);
                        let event_loop_proxy = Arc::clone(&event_loop_proxy);
                        std::thread::spawn(move || {
                            let error = match std::fs::read_to_string(&path) {
                                Ok(sql) => import_sql(&state, &sql).err(),
                                Err(error) => {
                                    Some(format!("Failed to read {}: {error}", path.display()))
                                }
                            };
                            event_loop_proxy.send_user_event(
                                serde_json::to_string(&IpcMessage::ImportSqlResponse {
                                    request_id,
                                    cancelled: false,
                                    error,
                                })
                                .expect("Failed to serialize response"),
                            );
                        });
                    } else {
                        webview.send_ipc_message(
                            serde_json::to_string(&IpcMessage::ImportSqlResponse {
                                request_id,
                                cancelled: true,
                                error: None,
                            })
                            .expect("Failed to serialize response"),
                        );
                    }
                }
                IpcMessage::ExportSql {
                    request_id,
                    file_name,
                } => {
                    let path = FileDialog::new()
                        .parent(&window)
                        .title("Export SQL")
                        .file_name(file_name)
                        .add_filter("SQL files", &["sql"])
                        .save_file();
                    if let Some(mut path) = path {
                        if path.extension().is_none() {
                            path.set_extension("sql");
                        }
                        let state = Arc::clone(&state);
                        let event_loop_proxy = Arc::clone(&event_loop_proxy);
                        std::thread::spawn(move || {
                            let error = export_sql(&state)
                                .and_then(|sql| {
                                    std::fs::write(&path, sql).map_err(|error| {
                                        format!("Failed to write {}: {error}", path.display())
                                    })
                                })
                                .err();
                            event_loop_proxy.send_user_event(
                                serde_json::to_string(&IpcMessage::ExportSqlResponse {
                                    request_id,
                                    cancelled: false,
                                    error,
                                })
                                .expect("Failed to serialize response"),
                            );
                        });
                    } else {
                        webview.send_ipc_message(
                            serde_json::to_string(&IpcMessage::ExportSqlResponse {
                                request_id,
                                cancelled: true,
                                error: None,
                            })
                            .expect("Failed to serialize response"),
                        );
                    }
                }
                IpcMessage::OpenMysql {
                    request_id,
                    transport,
                    host,
                    port,
                    socket,
                    user,
                    password,
                    tls,
                    remember,
                    previous_connection,
                } => {
                    if mysql_connection_pending.swap(true, Ordering::AcqRel) {
                        webview.send_ipc_message(
                            serde_json::to_string(&IpcMessage::OpenMysqlResponse {
                                request_id,
                                ok: false,
                                error: Some(
                                    "A MySQL connection is already being opened".to_string(),
                                ),
                                credential_saved: false,
                                credential_error: None,
                            })
                            .expect("Failed to serialize response"),
                        );
                        return;
                    }
                    let request = OpenMysqlRequest {
                        request_id,
                        connection_generation: connection_generation.fetch_add(1, Ordering::AcqRel)
                            + 1,
                        transport,
                        host,
                        port,
                        socket,
                        user,
                        password,
                        tls,
                        remember,
                        previous_connection,
                    };
                    let state = Arc::clone(&state);
                    let event_loop_proxy = Arc::clone(&event_loop_proxy);
                    let mysql_connection_pending = Arc::clone(&mysql_connection_pending);
                    let connection_generation = Arc::clone(&connection_generation);
                    std::thread::spawn(move || {
                        let _pending_guard = MysqlConnectionPendingGuard(mysql_connection_pending);
                        let request_id = request.request_id;
                        let response =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                open_mysql(request, &state, &connection_generation)
                            }))
                            .unwrap_or_else(|_| {
                                IpcMessage::OpenMysqlResponse {
                                    request_id,
                                    ok: false,
                                    error: Some("Failed to open MySQL connection".to_string()),
                                    credential_saved: false,
                                    credential_error: None,
                                }
                            });
                        event_loop_proxy.send_user_event(
                            serde_json::to_string(&response)
                                .expect("Failed to serialize MySQL response"),
                        );
                    });
                }
                IpcMessage::SelectMysqlDatabase {
                    request_id,
                    database,
                } => {
                    let request_generation =
                        connection_generation.fetch_add(1, Ordering::AcqRel) + 1;
                    let settings = state.lock().expect("mutex poisoned").mysql_settings();
                    let result = settings
                        .ok_or_else(|| "No MySQL connection open".to_string())
                        .and_then(|settings| {
                            settings
                                .connect(Some(&database))
                                .map(|connection| (connection, settings))
                        });
                    let (ok, error) = match result {
                        Ok((connection, settings)) => {
                            if replace_database_state_if_current(
                                &state,
                                &connection_generation,
                                request_generation,
                                DatabaseState::mysql(connection, settings, database),
                            ) {
                                (true, None)
                            } else {
                                (false, Some("Connection request was superseded".to_string()))
                            }
                        }
                        Err(error) => (false, Some(error)),
                    };
                    webview.send_ipc_message(
                        serde_json::to_string(&IpcMessage::SelectMysqlDatabaseResponse {
                            request_id,
                            ok,
                            error,
                        })
                        .expect("Failed to serialize response"),
                    );
                }
                _ => {}
            }
        }
        Event::UserEvent(message) => webview.send_ipc_message(message),
        _ => {}
    });
}
