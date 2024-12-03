/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use garde::Validate;
use http::{Method, Request, Response, Status};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const HTTP_PORT: u16 = 8000;

#[derive(Clone)]
struct Context {
    database: Arc<sqlite::Connection>,
}

// MARK: Routes
fn home(_: &Request, _: Response, _: Context) -> Result<Response> {
    Ok(Response::new().body(concat!("Persons v", env!("CARGO_PKG_VERSION"))))
}

fn not_found(_: &Request, _: Response, _: Context) -> Result<Response> {
    Ok(Response::new()
        .status(Status::NotFound)
        .body("404 Not Found"))
}

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
    let res = Response::new().header("Access-Control-Allow-Origin", "*");

    if req.path == "/" {
        return home(req, res, ctx);
    }

    if req.path.starts_with("/persons") {
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

    not_found(req, res, ctx)
}

// MARK: Database
fn open_database() -> Result<sqlite::Connection> {
    // Create new database
    let database = sqlite::Connection::open("database.db")?;
    database.execute(
        "CREATE TABLE IF NOT EXISTS persons(
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
    )?;

    // Insert persons
    let persons_count = database
        .query::<usize>("SELECT COUNT(id) FROM persons", ())?
        .next()
        .expect("Should be some")?;
    if persons_count == 0 {
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
    }

    Ok(database)
}

// MARK: Main
fn main() -> Result<()> {
    let ctx = Context {
        database: Arc::new(open_database()?),
    };
    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    http::serve_with_ctx(
        |req, ctx| match handler(req, ctx) {
            Ok(response) => response,
            Err(err) => {
                println!("\nError: {:?}", err);
                Response::new()
                    .status(Status::InternalServerError)
                    .body("500 Internal Server Error")
            }
        },
        HTTP_PORT,
        ctx,
    )
}
