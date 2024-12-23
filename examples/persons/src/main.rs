/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple persons REST API example

use std::net::{Ipv4Addr, TcpListener};

use chrono::{DateTime, Utc};
use http::{Method, Request, Response, Status};
use router::{Path, Router};
use serde::Deserialize;
use sqlite::{FromRow, FromValue};
use uuid::Uuid;
use validate::Validate;

mod api {
    include!(concat!(env!("OUT_DIR"), "/persons_api.rs"));
}

const HTTP_PORT: u16 = 8080;
const LIMIT_DEFAULT: i64 = 20;
const LIMIT_MAX: i64 = 50;

// MARK: Utils
fn validate_name(name: &str) -> validate::Result {
    if name.to_lowercase() == "bastiaan" {
        Err(validate::Error::new("name can't be Bastiaan"))
    } else {
        Ok(())
    }
}

// MARK: Context
#[derive(Clone)]
struct Context {
    database: sqlite::Connection,
}

// MARK: Layers
fn log_layer(req: &Request, _: &mut Context) -> Option<Response> {
    println!("{} {}", req.method, req.url.path);
    None
}

fn cors_pre_layer(req: &Request, _: &mut Context) -> Option<Response> {
    if req.method == Method::Options {
        Some(
            Response::with_header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, POST")
                .header("Access-Control-Max-Age", "86400"),
        )
    } else {
        None
    }
}

fn cors_post_layer(_: &Request, _: &mut Context, res: Response) -> Response {
    res.header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST")
        .header("Access-Control-Max-Age", "86400")
}

// MARK: Person
#[derive(Clone, FromRow)]
struct Person {
    id: Uuid,
    name: String,
    #[sqlite(rename = "age")]
    age_in_years: i64,
    relation: Relation,
    created_at: DateTime<Utc>,
}

impl Default for Person {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: String::default(),
            age_in_years: 0,
            relation: Relation::Me,
            created_at: Utc::now(),
        }
    }
}

impl From<Person> for api::Person {
    fn from(person: Person) -> Self {
        Self {
            id: person.id,
            name: person.name,
            age_in_years: person.age_in_years,
            relation: person.relation.into(),
            is_adult: Some(person.age_in_years >= 18),
            created_at: person.created_at,
        }
    }
}

#[derive(Copy, Clone, FromValue)]
enum Relation {
    Me = 0,
    Brother = 1,
    Sister = 2,
}

impl From<api::Relation> for Relation {
    fn from(relation: api::Relation) -> Self {
        match relation {
            api::Relation::Me => Relation::Me,
            api::Relation::Brother => Relation::Brother,
            api::Relation::Sister => Relation::Sister,
        }
    }
}

impl From<Relation> for api::Relation {
    fn from(relation: Relation) -> Self {
        match relation {
            Relation::Me => api::Relation::Me,
            Relation::Brother => api::Relation::Brother,
            Relation::Sister => api::Relation::Sister,
        }
    }
}

// MARK: Routes
fn home(_: &Request, _: &Context, _: &Path) -> Response {
    Response::with_body(concat!("Persons v", env!("CARGO_PKG_VERSION")))
}

fn not_found(_: &Request, _: &Context, _: &Path) -> Response {
    Response::with_status(Status::NotFound).body("404 Not found")
}

#[derive(Default, Deserialize, Validate)]
struct IndexQuery {
    #[serde(rename = "q")]
    query: Option<String>,
    #[validate(range(min = 1))]
    page: Option<i64>,
    #[validate(range(min = 1, max = LIMIT_MAX))]
    limit: Option<i64>,
}

fn persons_index(req: &Request, ctx: &Context, _: &Path) -> Response {
    // Parse request query
    let query = match req.url.query.as_ref() {
        Some(query) => match serde_urlencoded::from_str::<IndexQuery>(query) {
            Ok(query) => query,
            Err(_) => return Response::with_status(Status::BadRequest),
        },
        None => IndexQuery::default(),
    };
    if let Err(report) = query.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Get or search persons
    let limit = query.limit.unwrap_or(LIMIT_DEFAULT);
    let persons = ctx
        .database
        .query::<Person>(
            format!(
                "SELECT {} FROM persons WHERE name LIKE ? LIMIT ? OFFSET ?",
                Person::columns()
            ),
            (
                format!("%{}%", query.query.unwrap_or_default().replace("%", "\\%")),
                limit,
                (query.page.unwrap_or(1) - 1) * limit,
            ),
        )
        .map(Into::<api::Person>::into)
        .collect::<Vec<_>>();

    // Return persons
    Response::with_json(persons)
}

#[derive(Deserialize, Validate)]
struct PersonCreateUpdateBody {
    #[validate(ascii, length(min = 3, max = 25), custom(validate_name))]
    name: String,
    #[validate(range(min = 8))]
    age_in_years: i64,
    relation: api::Relation,
}

fn persons_create(req: &Request, ctx: &Context, _: &Path) -> Response {
    // Parse and validate body
    let body = match serde_urlencoded::from_str::<PersonCreateUpdateBody>(&req.body) {
        Ok(body) => body,
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Create person
    let person = Person {
        name: body.name,
        age_in_years: body.age_in_years,
        relation: body.relation.into(),
        ..Default::default()
    };
    ctx.database.execute(
        format!(
            "INSERT INTO persons ({}) VALUES ({})",
            Person::columns(),
            Person::values()
        ),
        person.clone(),
    );

    // Return created person
    Response::with_json(Into::<api::Person>::into(person))
}

fn get_person(ctx: &Context, path: &Path) -> Option<Person> {
    // Parse person id from url
    let person_id = match path
        .get("person_id")
        .expect("Should be some")
        .parse::<Uuid>()
    {
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

fn persons_show(req: &Request, ctx: &Context, path: &Path) -> Response {
    // Get person
    let person = match get_person(ctx, path) {
        Some(person) => person,
        None => return not_found(req, ctx, path),
    };

    // Return person
    Response::with_json(Into::<api::Person>::into(person))
}

fn persons_update(req: &Request, ctx: &Context, path: &Path) -> Response {
    // Get person
    let mut person = match get_person(ctx, path) {
        Some(person) => person,
        None => return not_found(req, ctx, path),
    };

    // Parse and validate body
    let body = match serde_urlencoded::from_str::<PersonCreateUpdateBody>(&req.body) {
        Ok(body) => body,
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Update person
    person.name = body.name;
    person.age_in_years = body.age_in_years;
    person.relation = body.relation.into();
    ctx.database.execute(
        "UPDATE persons SET name = ?, age = ? WHERE id = ? LIMIT 1",
        (person.name.clone(), person.age_in_years, person.id),
    );

    // Return updated person
    Response::with_json(Into::<api::Person>::into(person))
}

fn persons_delete(req: &Request, ctx: &Context, path: &Path) -> Response {
    // Get person
    let person = match get_person(ctx, path) {
        Some(person) => person,
        None => return not_found(req, ctx, path),
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
            relation INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        )",
        (),
    );

    // Insert persons
    let persons_count = database
        .query::<i64>("SELECT COUNT(id) FROM persons", ())
        .next()
        .expect("Should be some");
    if persons_count == 0 {
        let persons = vec![
            Person {
                name: "Bastiaan".to_string(),
                age_in_years: 20,
                relation: Relation::Me,
                ..Default::default()
            },
            Person {
                name: "Sander".to_string(),
                age_in_years: 19,
                relation: Relation::Brother,
                ..Default::default()
            },
            Person {
                name: "Leonard".to_string(),
                age_in_years: 16,
                relation: Relation::Brother,
                ..Default::default()
            },
            Person {
                name: "Jiska".to_string(),
                age_in_years: 14,
                relation: Relation::Sister,
                ..Default::default()
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

    let router = Router::<Context>::with(ctx)
        .pre_layer(log_layer)
        .pre_layer(cors_pre_layer)
        .post_layer(cors_post_layer)
        .get("/", home)
        .get("/persons", persons_index)
        .post("/persons", persons_create)
        .get("/persons/:person_id", persons_show)
        .put("/persons/:person_id", persons_update)
        .delete("/persons/:person_id", persons_delete)
        .fallback(not_found)
        .build();

    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, HTTP_PORT))
        .unwrap_or_else(|_| panic!("Can't bind to port: {}", HTTP_PORT));
    http::serve(listener, move |req| router.handle(req));
}
