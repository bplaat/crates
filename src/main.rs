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
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::{Method, Request, Response, Status};

mod http;
mod sqlite;

const HTTP_PORT: u16 = 8082;

#[derive(Clone)]
struct Context {
    database: Arc<sqlite::Connection>,
}

// MARK: Persons
#[derive(Deserialize, Serialize)]
struct Person {
    id: Uuid,
    name: String,
    age: i32,
    created_at: DateTime<Utc>,
}

fn persons_index(_: &Request, res: Response, ctx: Context) -> Result<Response> {
    // Get persons
    let persons = ctx
        .database
        .query::<Person>("SELECT id, name, age, created_at FROM persons", ())?
        .collect::<Result<Vec<_>>>()?;
    Ok(res.json(persons))
}

fn persons_create(req: &Request, res: Response, ctx: Context) -> Result<Response> {
    // Parse and validate body
    #[derive(Deserialize, Validate)]
    struct PersonsCreateBody {
        #[garde(ascii, length(min = 3, max = 25))]
        name: String,
        #[garde(range(min = 8))]
        age: i32,
    }
    let body = match serde_urlencoded::from_str::<PersonsCreateBody>(&req.body) {
        Ok(body) => body,
        Err(_) => {
            return Ok(res.status(Status::BadRequest).body("400 Bad Request"));
        }
    };
    if let Err(err) = body.validate() {
        return Ok(res.status(Status::BadRequest).json(err));
    }

    // Create person
    let person = Person {
        id: Uuid::now_v7(),
        name: body.name,
        age: body.age,
        created_at: Utc::now(),
    };
    ctx.database
        .query::<()>(
            "INSERT INTO persons (id, name, age, created_at) VALUES (?, ?, ?, ?)",
            &person,
        )?
        .next();

    Ok(res.json(person))
}

fn persons_show(_: &Request, res: Response, ctx: Context, person_id: Uuid) -> Result<Response> {
    // Get person
    let person = ctx
        .database
        .query::<Person>(
            "SELECT id, name, age, created_at FROM persons WHERE id = ? LIMIT 1",
            person_id,
        )?
        .next();

    if let Some(Ok(person)) = person {
        Ok(res.json(person))
    } else {
        Ok(res.status(Status::NotFound).body("404 Not Found"))
    }
}

// MARK: Handler
fn handler(req: &Request, ctx: Context) -> Result<Response> {
    println!("{} {}", req.method, req.path);

    if req.path == "/" {
        return Ok(Response::new().html("<h1>Hello World!</h1>"));
    }

    if req.path == "/greet" {
        if req.method != Method::Post {
            return Ok(Response::new()
                .status(Status::MethodNotAllowed)
                .body("405 Method Not Allowed"));
        }

        #[derive(Deserialize)]
        struct GreetBody {
            name: String,
        }
        let body = match serde_urlencoded::from_str::<GreetBody>(&req.body) {
            Ok(body) => body,
            Err(_) => {
                return Ok(Response::new()
                    .status(Status::BadRequest)
                    .body("400 Bad Request"));
            }
        };
        return Ok(Response::new().html(format!("<h1>Hello {}!</h1>", body.name)));
    }

    if req.path == "/redirect" {
        return Ok(Response::new().redirect("/"));
    }

    if req.path == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        return Ok(Response::new().html("<h1>Sleeping done!</h1>"));
    }

    // REST API example
    if req.path.starts_with("/persons") {
        let res = Response::new().header("Access-Control-Allow-Origin", "*");
        if req.path == "/persons" {
            if req.method == Method::Post {
                return persons_create(req, res, ctx);
            }
            return persons_index(req, res, ctx);
        }
        if req.path.starts_with("/persons/") {
            // Parse person id from url
            let person_id = match req.path["/persons/".len()..].parse::<Uuid>() {
                Ok(id) => id,
                Err(_) => {
                    return Ok(res.status(Status::BadRequest).body("400 Bad Request"));
                }
            };
            return persons_show(req, res, ctx, person_id);
        }
    }

    Ok(Response::new()
        .status(Status::NotFound)
        .html("<h1>404 Not Found</h1>"))
}

// MARK: Database
fn open_database() -> Result<sqlite::Connection> {
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

    // Insert persons
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
    Ok(database)
}

// MARK: Main
fn main() -> Result<()> {
    let ctx = Context {
        database: Arc::new(open_database()?),
    };
    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    http::serve_with_ctx(handler, HTTP_PORT, ctx)
}
