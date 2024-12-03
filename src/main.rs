/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::Arc;
use std::time::Duration;
use std::{fs, thread};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::{Method, Request, Response, Status};

mod http;
mod sqlite;

#[derive(Clone, Deserialize, Serialize)]
struct Person {
    id: Uuid,
    name: String,
    age: i64,
    created_at: DateTime<Utc>,
}

#[derive(Clone)]
struct Context {
    database: Arc<sqlite::Connection>,
}

#[derive(Deserialize)]
struct GreetBody {
    name: String,
}

const HTTP_PORT: u16 = 8082;

fn handler(req: Request, ctx: Context) -> Response {
    println!("{} {}", req.method, req.path);

    if req.path == "/" {
        return Response::new().html("<h1>Hello World!</h1>");
    }

    if req.path == "/greet" {
        if req.method == Method::Post {
            let body = match serde_urlencoded::from_str::<GreetBody>(&req.body) {
                Ok(body) => body,
                Err(_) => {
                    return Response::new()
                        .status(Status::BadRequest)
                        .body("400 Bad Request");
                }
            };
            return Response::new().html(format!("<h1>Hello {}!</h1>", body.name));
        }
        return Response::new()
            .status(Status::MethodNotAllowed)
            .body("405 Method Not Allowed");
    }

    if req.path == "/redirect" {
        return Response::new().redirect("/");
    }

    if req.path == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        return Response::new().html("<h1>Sleeping done!</h1>");
    }

    // REST API example
    if req.path.starts_with("/persons") {
        let res = Response::new().header("Access-Control-Allow-Origin", "*");

        if req.path == "/persons" {
            let persons = ctx
                .database
                .query::<Person>("SELECT id, name, age, created_at FROM persons", ())
                .expect("Can't run query")
                .map(|person| person.expect("Can't query person"))
                .collect::<Vec<Person>>();
            return res.json(persons);
        }

        if req.path.starts_with("/persons/") {
            let person_id = match req.path["/persons/".len()..].parse::<Uuid>() {
                Ok(id) => id,
                Err(_) => {
                    return res.status(Status::BadRequest).body("400 Bad Request");
                }
            };

            let person = ctx
                .database
                .query::<Person>(
                    "SELECT id, name, age, created_at FROM persons WHERE id = ? LIMIT 1",
                    person_id,
                )
                .expect("Can't run query")
                .next();
            if let Some(Ok(person)) = person {
                return res.json(person);
            }
        }

        return res.status(Status::NotFound).body("404 Not Found");
    }

    Response::new()
        .status(Status::NotFound)
        .html("<h1>404 Not Found</h1>")
}

fn main() -> Result<()> {
    // Remove old database
    fs::remove_file("database.db")?;

    // Create new database
    let database = sqlite::Connection::open("database.db")?;
    database.execute(
        "CREATE TABLE persons (
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
    )?;

    let persons = vec![
        Person {
            id: Uuid::now_v7(),
            name: "Bastiaan".to_string(),
            age: 20,
            created_at: Utc::now(),
        },
        Person {
            id: Uuid::now_v7(),
            name: "Sander".to_string(),
            age: 19,
            created_at: Utc::now(),
        },
        Person {
            id: Uuid::now_v7(),
            name: "Leonard".to_string(),
            age: 16,
            created_at: Utc::now(),
        },
        Person {
            id: Uuid::now_v7(),
            name: "Jiska".to_string(),
            age: 14,
            created_at: Utc::now(),
        },
    ];
    for person in &persons {
        database
            .query::<()>(
                "INSERT INTO persons (id, name, age, created_at) VALUES (?, ?, ?, ?)",
                person,
            )?
            .next();
    }

    // Start server
    let ctx = Context {
        database: Arc::new(database),
    };
    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    http::serve_with_ctx(handler, HTTP_PORT, ctx)
}
