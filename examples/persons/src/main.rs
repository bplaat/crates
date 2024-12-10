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
    Response::with_body(concat!("Persons v", env!("CARGO_PKG_VERSION")))
}

fn not_found(_: &Request, _: &Context, _: &Path) -> Response {
    Response::with_status(Status::NotFound)
}

#[derive(Clone, Serialize, FromRow)]
struct Person {
    id: Uuid,
    name: String,
    age: i64,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate)]
struct PersonBody {
    #[validate(ascii, length(min = 3, max = 25), custom(validate_name))]
    name: String,
    #[validate(range(min = 8))]
    age: i64,
}

fn persons_index(req: &Request, ctx: &Context, _: &Path) -> Response {
    // Parse request query
    #[derive(Deserialize)]
    struct Query {
        #[serde(rename = "q")]
        query: Option<String>,
    }
    let query = match req.url.query.as_ref() {
        Some(query) => match serde_urlencoded::from_str::<Query>(query) {
            Ok(query) => query,
            Err(_) => return Response::with_status(Status::BadRequest),
        },
        None => Query { query: None },
    };

    // Get or search persons
    let persons = if let Some(query) = query.query {
        ctx.database.query::<Person>(
            format!(
                "SELECT {} FROM persons WHERE name LIKE ?",
                Person::columns()
            ),
            format!("%{}%", query),
        )
    } else {
        ctx.database
            .query::<Person>(format!("SELECT {} FROM persons", Person::columns()), ())
    }
    .collect::<Vec<_>>();

    // Persons response
    Response::with_json(persons)
}

fn persons_create(req: &Request, ctx: &Context, _: &Path) -> Response {
    // Parse and validate body
    let body = match serde_urlencoded::from_str::<PersonBody>(&req.body) {
        Ok(body) => body,
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(errors) = body.validate() {
        return Response::with_status(Status::BadRequest).json(errors);
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
        person.clone(),
    );

    // Created person response
    Response::with_json(person)
}

fn get_person(ctx: &Context, path: &Path) -> Option<Person> {
    // Parse person id from url
    let person_id = match path.get("person_id").unwrap().parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return None,
    };

    // Get person
    ctx.database
        .query::<Person>(
            format!(
                "SELECT {} FROM persons WHERE id = ? LIMIT 1",
                Person::columns()
            ),
            person_id,
        )
        .next()
}

fn persons_show(_: &Request, ctx: &Context, path: &Path) -> Response {
    // Get person
    let person = match get_person(ctx, path) {
        Some(person) => person,
        None => return Response::with_status(Status::NotFound),
    };

    // Person response
    Response::with_json(person)
}

fn persons_update(req: &Request, ctx: &Context, path: &Path) -> Response {
    // Get person
    let mut person = match get_person(ctx, path) {
        Some(person) => person,
        None => return Response::with_status(Status::NotFound),
    };

    // Parse and validate body
    let body = match serde_urlencoded::from_str::<PersonBody>(&req.body) {
        Ok(body) => body,
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(errors) = body.validate() {
        return Response::with_status(Status::BadRequest).json(errors);
    }

    // Update person
    person.name = body.name;
    person.age = body.age;
    ctx.database.execute(
        "UPDATE persons SET name = ?, age = ? WHERE id = ? LIMIT 1",
        (person.name.clone(), person.age, person.id),
    );

    // Updated person response
    Response::with_json(person)
}

fn persons_delete(_: &Request, ctx: &Context, path: &Path) -> Response {
    // Get person
    let person = match get_person(ctx, path) {
        Some(person) => person,
        None => {
            return Response::with_status(Status::NotFound);
        }
    };

    // Delete person
    ctx.database
        .execute("DELETE FROM persons WHERE id = ?", person.id);

    // Success response
    Response::new()
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
            .put("/persons/:person_id", persons_update)
            .delete("/persons/:person_id", persons_delete)
            .fallback(not_found),
    );

    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, HTTP_PORT))
        .unwrap_or_else(|_| panic!("Can't bind to port: {}", HTTP_PORT));
    http::serve(listener, move |req| {
        println!("{} {}", req.method, req.url.path);

        // Cors middleware
        if req.method == Method::Options {
            return Response::with_header("Access-Control-Allow-Origin", "*")
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
