/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use garde::Validate;
use http::{Request, Response, Status};
use router::{Path, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const HTTP_PORT: u16 = 8000;

#[derive(Clone)]
struct Context {
    database: Arc<sqlite::Connection>,
}

// MARK: Routes
fn home(_: &Request, _: &Context, _: &Path) -> Result<Response> {
    Ok(Response::new().body(concat!("Persons v", env!("CARGO_PKG_VERSION"))))
}

fn not_found(_: &Request, _: &Context, _: &Path) -> Result<Response> {
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

fn persons_index(_: &Request, ctx: &Context, _: &Path) -> Result<Response> {
    // Get persons
    let persons = ctx
        .database
        .query::<Person>("SELECT id, name, age, created_at FROM persons", ())?
        .collect::<Result<Vec<_>>>()?;
    Ok(Response::new().json(persons))
}

fn persons_create(req: &Request, ctx: &Context, _: &Path) -> Result<Response> {
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
            return Ok(Response::new()
                .status(Status::BadRequest)
                .body("400 Bad Request"));
        }
    };
    if let Err(err) = body.validate() {
        return Ok(Response::new().status(Status::BadRequest).json(err));
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

    Ok(Response::new().json(person))
}

fn persons_show(_: &Request, ctx: &Context, path: &Path) -> Result<Response> {
    // Parse person id from url
    let person_id = match path
        .get("person_id")
        .expect("Should be some")
        .parse::<Uuid>()
    {
        Ok(id) => id,
        Err(_) => {
            return Ok(Response::new()
                .status(Status::BadRequest)
                .body("400 Bad Request"));
        }
    };

    // Get person
    let person = ctx
        .database
        .query::<Person>(
            "SELECT id, name, age, created_at FROM persons WHERE id = ? LIMIT 1",
            person_id,
        )?
        .next();

    if let Some(Ok(person)) = person {
        Ok(Response::new().json(person))
    } else {
        Ok(Response::new()
            .status(Status::NotFound)
            .body("404 Not Found"))
    }
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

    let router = Arc::new(
        Router::<Context>::new()
            .get("/", home)
            .get("/persons", persons_index)
            .post("/persons", persons_create)
            .get("/persons/:person_id", persons_show)
            .fallback(not_found),
    );

    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    http::serve(
        move |req| {
            // Error middleware
            let res = match router.next(req, &ctx) {
                Ok(res) => res,
                Err(err) => {
                    println!("Error: {:?}", err);
                    Response::new()
                        .status(http::Status::InternalServerError)
                        .body("500 Internal Server Error")
                }
            };

            // Cors middleware
            res.header("Access-Control-Allow-Origin", "*")
        },
        HTTP_PORT,
    )
}
