/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use http::{Method, Request, Response, Status};
use router::{Path, Router};
use serde::{Deserialize, Serialize};
use sqlite::FromRow;
use uuid::Uuid;
use validate::Validate;

const HTTP_PORT: u16 = 8080;

#[derive(Clone)]
struct Context {
    database: sqlite::Connection,
}

fn validate_name(name: &str) -> validate::Result {
    if name.to_lowercase() == "bastiaan" {
        Err(validate::Error::new("name can't be Bastiaan"))
    } else {
        Ok(())
    }
}

// MARK: Routes
fn home(_: &Request, _: &Context, _: &Path) -> Response {
    Response::new().body(concat!("Persons v", env!("CARGO_PKG_VERSION")))
}

fn not_found(_: &Request, _: &Context, _: &Path) -> Response {
    Response::new()
        .status(Status::NotFound)
        .body("404 Not Found")
}

#[derive(Clone, Serialize, FromRow)]
struct Person {
    id: Uuid,
    name: String,
    age: i64,
    created_at: DateTime<Utc>,
}

fn persons_index(_: &Request, ctx: &Context, _: &Path) -> Response {
    // Get persons
    let persons = ctx
        .database
        .query::<Person>(format!("SELECT {} FROM persons", Person::columns()), ())
        .collect::<Vec<_>>();
    Response::new().json(persons)
}

fn persons_create(req: &Request, ctx: &Context, _: &Path) -> Response {
    // Parse and validate body
    #[derive(Deserialize, Validate)]
    struct Body {
        #[validate(ascii, length(min = 3, max = 25), custom(validate_name))]
        name: String,
        #[validate(range(min = 8))]
        age: i64,
    }
    let body = match serde_urlencoded::from_str::<Body>(&req.body) {
        Ok(body) => body,
        Err(_) => {
            return Response::new()
                .status(Status::BadRequest)
                .body("400 Bad Request");
        }
    };
    if let Err(errors) = body.validate() {
        return Response::new().status(Status::BadRequest).json(errors);
    }

    // Create person
    let person = Person {
        id: Uuid::now_v7(),
        name: body.name,
        age: body.age,
        created_at: Utc::now(),
    };
    ctx.database.execute(
        format!(
            "INSERT INTO persons ({}) VALUES ({})",
            Person::columns(),
            Person::values()
        ),
        (
            person.id,
            person.name.clone(),
            person.age,
            person.created_at,
        ),
    );

    Response::new().json(person)
}

fn persons_show(_: &Request, ctx: &Context, path: &Path) -> Response {
    // Parse person id from url
    let person_id = match path.get("person_id").unwrap().parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return Response::new()
                .status(Status::BadRequest)
                .body("400 Bad Request");
        }
    };

    // Get person
    let person = ctx
        .database
        .query::<Person>(
            format!(
                "SELECT {} FROM persons WHERE id = ? LIMIT 1",
                Person::columns()
            ),
            person_id,
        )
        .next();

    if let Some(person) = person {
        Response::new().json(person)
    } else {
        Response::new()
            .status(Status::NotFound)
            .body("404 Not Found")
    }
}

// MARK: Database
fn open_database() -> Result<sqlite::Connection, sqlite::ConnectionError> {
    // Create new database
    let database = sqlite::Connection::open("database.db")?;
    database.execute(
        "CREATE TABLE IF NOT EXISTS persons(
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        (),
    );

    // Insert persons
    let persons_count = database
        .query::<i64>("SELECT COUNT(id) FROM persons", ())
        .next()
        .unwrap();
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
        for person in persons {
            database.execute(
                format!(
                    "INSERT INTO persons ({}) VALUES ({})",
                    Person::columns(),
                    Person::values()
                ),
                person,
            );
        }
    }

    Ok(database)
}

// MARK: Main
fn main() {
    let ctx = Context {
        database: open_database().expect("Can't open database"),
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
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, HTTP_PORT))
        .unwrap_or_else(|_| panic!("Can't bind to port: {}", HTTP_PORT));
    http::serve(listener, move |req| {
        println!("{} {}", req.method, req.path);

        // Cors middleware
        if req.method == Method::Options {
            return Response::new()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, POST")
                .header("Access-Control-Max-Age", "86400");
        }

        // Router
        let res = router.next(req, &ctx);

        // Cors middleware
        res.header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST")
            .header("Access-Control-Max-Age", "86400")
    });
}
