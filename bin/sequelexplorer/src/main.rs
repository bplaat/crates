/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use bsqlite::{Connection, OpenMode, Value};
use bwebview::{Event, EventLoopBuilder, LogicalSize, WebviewBuilder};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use serde_json::json;
use small_http::{Request, Response, Status};
use small_router::RouterBuilder;
use url::Url;

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

// MARK: State
type State = Arc<Mutex<Option<Connection>>>;

// MARK: Open
#[derive(Deserialize)]
struct OpenBody {
    path: String,
}

fn db_open(req: &Request, state: &State) -> Response {
    let body: OpenBody = match serde_json::from_slice(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(e) => return Response::with_json(json!({ "error": e.to_string() })),
    };
    match Connection::open(&body.path, OpenMode::ReadOnly) {
        Ok(conn) => {
            *state.lock().expect("mutex poisoned") = Some(conn);
            Response::with_json(json!({ "ok": true }))
        }
        Err(e) => Response::with_json(json!({ "error": e.to_string() })),
    }
}

// MARK: Tables
fn db_tables(_req: &Request, state: &State) -> Response {
    let guard = state.lock().expect("mutex poisoned");
    let Some(conn) = guard.as_ref() else {
        return Response::with_json(json!({ "error": "No database open" }));
    };

    let table_names: Vec<String> = conn
        .query::<String>(
            "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
            (),
        )
        .collect();
    Response::with_json(&table_names)
}

// MARK: Table data
fn column_type_to_string(r#type: bsqlite::ColumnType) -> String {
    match r#type {
        bsqlite::ColumnType::Null => "NULL".to_string(),
        bsqlite::ColumnType::Integer => "INTEGER".to_string(),
        bsqlite::ColumnType::Float => "FLOAT".to_string(),
        bsqlite::ColumnType::Text => "TEXT".to_string(),
        bsqlite::ColumnType::Blob => "BLOB".to_string(),
    }
}

fn column_value_to_json(value: Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Integer(i) => json!(i),
        Value::Real(f) => json!(f),
        Value::Text(s) => json!(s),
        Value::Blob(b) => {
            if let Ok(uuid) = uuid::Uuid::from_slice(&b) {
                json!(uuid.to_string())
            } else {
                json!(format!("<BLOB {} bytes>", b.len()))
            }
        }
    }
}

#[derive(Serialize)]
struct ColumnInfo {
    name: String,
    r#type: String,
}

#[derive(Serialize)]
struct TableData {
    columns: Vec<ColumnInfo>,
    rows: Vec<Vec<serde_json::Value>>,
    total: i64,
}

fn parse_query_param(url: &Url, key: &str) -> Option<String> {
    let query = url.query()?;
    query.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next()?;
        let v = parts.next().unwrap_or("");
        if k == key { Some(v.to_string()) } else { None }
    })
}

fn db_table_data(req: &Request, state: &State) -> Response {
    let name = req
        .params
        .get("name")
        .expect("name param should be present");
    let offset: i64 = parse_query_param(&req.url, "offset")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let limit: i64 = parse_query_param(&req.url, "limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let guard = state.lock().expect("mutex poisoned");
    let Some(conn) = guard.as_ref() else {
        return Response::with_json(json!({ "error": "No database open" }));
    };

    let total: i64 = conn.query_some::<i64>(&format!("SELECT COUNT(*) FROM \"{name}\""), ());

    let query = format!("SELECT * FROM \"{name}\" LIMIT {limit} OFFSET {offset}");
    let mut stmt = conn.prepare::<()>(&query);

    let col_count = stmt.column_count();
    let columns = (0..col_count)
        .map(|j| ColumnInfo {
            name: stmt.column_name(j),
            r#type: stmt
                .column_declared_type(j)
                .expect("Declared type should be present"),
        })
        .collect::<Vec<_>>();

    let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();
    while stmt.step().is_some() {
        let mut row = Vec::new();
        for i in 0..col_count {
            row.push(column_value_to_json(stmt.column_value(i)));
        }
        rows.push(row);
    }

    Response::with_json(&TableData {
        columns,
        rows,
        total,
    })
}

// MARK: Table schema
#[derive(Serialize)]
struct TableSchema {
    sql: String,
}

fn db_table_schema(req: &Request, state: &State) -> Response {
    let name = req
        .params
        .get("name")
        .expect("name param should be present");

    let guard = state.lock().expect("mutex poisoned");
    let Some(conn) = guard.as_ref() else {
        return Response::with_json(json!({ "error": "No database open" }));
    };

    let sql = conn
        .query::<String>(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?",
            name.to_string(),
        )
        .next();

    match sql {
        Some(sql) => {
            let sql = sql.replace("   ", " ").replace("\n    )", "\n)");
            Response::with_json(&TableSchema { sql })
        }
        None => Response::with_json(json!({ "error": "Table not found" })),
    }
}

// MARK: Custom query
#[derive(Deserialize)]
struct QueryBody {
    sql: String,
}

#[derive(Serialize)]
struct QueryResult {
    columns: Vec<ColumnInfo>,
    rows: Vec<Vec<serde_json::Value>>,
}

fn db_query(req: &Request, state: &State) -> Response {
    let body: QueryBody = match serde_json::from_slice(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(e) => return Response::with_json(json!({ "error": e.to_string() })),
    };

    let guard = state.lock().expect("mutex poisoned");
    let Some(conn) = guard.as_ref() else {
        return Response::with_json(json!({ "error": "No database open" }));
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut stmt = conn.prepare::<()>(&body.sql);

        let mut columns = None;
        let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();
        while stmt.step().is_some() {
            let col_count = stmt.column_count();
            if columns.is_none() {
                columns = Some(
                    (0..col_count)
                        .map(|j| ColumnInfo {
                            name: stmt.column_name(j),
                            r#type: column_type_to_string(stmt.column_type(j)),
                        })
                        .collect(),
                );
            }

            let mut row = Vec::new();
            for i in 0..col_count {
                row.push(column_value_to_json(stmt.column_value(i)));
            }
            rows.push(row);
        }

        QueryResult {
            columns: columns.unwrap_or_default(),
            rows,
        }
    }));

    match result {
        Ok(qr) => Response::with_json(&qr),
        Err(_) => Response::with_json(json!({ "error": "Query failed" })),
    }
}

// MARK: Main

fn main() {
    let state: State = Arc::new(Mutex::new(None));
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "SequelExplorer")
        .build();

    let router = RouterBuilder::<State>::with(Arc::clone(&state))
        .post("/api/open", db_open)
        .get("/api/tables", db_tables)
        .get("/api/table/:name/data", db_table_data)
        .get("/api/table/:name/schema", db_table_schema)
        .post("/api/query", db_query)
        .build();

    #[allow(unused_mut)]
    let mut webview_builder = WebviewBuilder::new()
        .title("Sequel Explorer")
        .size(LogicalSize::new(1200.0, 768.0))
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
