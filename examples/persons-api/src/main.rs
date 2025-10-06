/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple persons REST API example

#![forbid(unsafe_code)]

use std::env;
use std::net::{Ipv4Addr, TcpListener};

use bsqlite::{Connection, FromRow, FromValue, execute_args, query_args};
use chrono::{DateTime, Utc};
use const_format::formatcp;
use from_enum::FromEnum;
use log::info;
use serde::Deserialize;
use small_http::{Method, Request, Response, Status};
use small_router::{Router, RouterBuilder};
use uuid::Uuid;
use validate::Validate;

mod api {
    include!(concat!(env!("OUT_DIR"), "/persons_api.rs"));
}

// MARK: Validators
mod validators {
    pub(crate) fn name(name: &str) -> validate::Result {
        if name.to_lowercase() == "bastiaan" {
            Err(validate::Error::new("name can't be Bastiaan"))
        } else {
            Ok(())
        }
    }
}

// MARK: Context
#[derive(Clone)]
struct Context {
    database: Connection,
}

impl Context {
    fn with_database(path: &str) -> Self {
        let database = Connection::open(path).expect("Can't open database");
        database.enable_wal_logging();
        database.apply_various_performance_settings();
        database_create_tables(&database);
        database_seed(&database);
        Self { database }
    }

    #[cfg(test)]
    fn with_test_database() -> Self {
        let database = Connection::open_memory().expect("Can't open database");
        database_create_tables(&database);
        Self { database }
    }
}

// MARK: Database
trait DatabaseHelpers {
    fn insert_person(&self, person: Person);
}
impl DatabaseHelpers for Connection {
    fn insert_person(&self, person: Person) {
        self.execute(
            formatcp!(
                "INSERT INTO persons ({}) VALUES ({})",
                Person::columns(),
                Person::values()
            ),
            person,
        );
    }
}

fn database_create_tables(database: &Connection) {
    database.execute(
        "CREATE TABLE IF NOT EXISTS persons(
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL,
            relation INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        ) STRICT",
        (),
    );
}

fn database_seed(database: &Connection) {
    // Insert persons
    if database.query_some::<i64>("SELECT COUNT(id) FROM persons", ()) == 0 {
        database.insert_person(Person {
            name: "Bastiaan".to_string(),
            age_in_years: 20,
            relation: Relation::Me,
            ..Default::default()
        });
        database.insert_person(Person {
            name: "Sander".to_string(),
            age_in_years: 19,
            relation: Relation::Brother,
            ..Default::default()
        });
        database.insert_person(Person {
            name: "Leonard".to_string(),
            age_in_years: 16,
            relation: Relation::Brother,
            ..Default::default()
        });
        database.insert_person(Person {
            name: "Jiska".to_string(),
            age_in_years: 14,
            relation: Relation::Sister,
            ..Default::default()
        });
    }
}

// MARK: Layers
mod layers {
    use super::*;

    pub(crate) fn log_pre_layer(req: &Request, _: &mut Context) -> Option<Response> {
        info!("{} {}", req.method, req.url.path());
        None
    }

    pub(crate) fn cors_pre_layer(req: &Request, _: &mut Context) -> Option<Response> {
        if req.method == Method::Options {
            Some(Response::new())
        } else {
            None
        }
    }

    pub(crate) fn cors_post_layer(_: &Request, _: &mut Context, res: Response) -> Response {
        res.header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE")
            .header("Access-Control-Max-Age", "86400")
    }
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

#[derive(Copy, Clone, FromEnum, FromValue)]
#[from_enum(api::Relation)]
enum Relation {
    Me = 0,
    Brother = 1,
    Sister = 2,
}

// MARK: Routes
fn home(_: &Request, _: &Context) -> Response {
    Response::with_body(concat!("Persons v", env!("CARGO_PKG_VERSION")))
}

fn not_found(_: &Request, _: &Context) -> Response {
    Response::with_status(Status::NotFound).body("404 Not found")
}

#[derive(Deserialize, Validate)]
#[serde(default)]
struct IndexQuery {
    #[serde(rename = "q")]
    query: String,
    #[validate(range(min = 1))]
    page: i64,
    #[validate(range(min = 1, max = 50))]
    limit: i64,
}

impl Default for IndexQuery {
    fn default() -> Self {
        Self {
            query: "".to_string(),
            page: 1,
            limit: 20,
        }
    }
}

fn persons_index(req: &Request, ctx: &Context) -> Response {
    // Parse request query
    let query = match req.url.query() {
        Some(query) => match serde_urlencoded::from_str::<IndexQuery>(query) {
            Ok(query) => query,
            Err(_) => return Response::with_status(Status::BadRequest),
        },
        None => IndexQuery::default(),
    };
    if let Err(report) = query.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Get persons
    let search_query = format!("%{}%", query.query.replace("%", "\\%"));
    let total = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM persons WHERE name LIKE ?",
        search_query.clone(),
    );
    let persons = query_args!(
        Person,
        ctx.database,
        formatcp!(
            "SELECT {} FROM persons WHERE name LIKE :search_query LIMIT :limit OFFSET :offset",
            Person::columns()
        ),
        Args {
            search_query: search_query,
            limit: query.limit,
            offset: (query.page - 1) * query.limit
        }
    )
    .map(Into::<api::Person>::into)
    .collect::<Vec<_>>();

    // Return persons
    Response::with_json(api::PersonIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: persons,
    })
}

#[derive(Validate)]
struct PersonCreateUpdateBody {
    #[validate(ascii, length(min = 3, max = 25), custom(validators::name))]
    name: String,
    #[validate(range(min = 8))]
    age_in_years: i64,
    relation: Relation,
}

impl From<api::PersonCreateUpdateBody> for PersonCreateUpdateBody {
    fn from(body: api::PersonCreateUpdateBody) -> Self {
        Self {
            name: body.name,
            age_in_years: body.age_in_years,
            relation: body.relation.into(),
        }
    }
}

fn persons_create(req: &Request, ctx: &Context) -> Response {
    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::PersonCreateUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<PersonCreateUpdateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Create person
    let person = Person {
        name: body.name,
        age_in_years: body.age_in_years,
        relation: body.relation,
        ..Default::default()
    };
    ctx.database.insert_person(person.clone());

    // Return created person
    Response::with_json(Into::<api::Person>::into(person))
}

fn get_person(req: &Request, ctx: &Context) -> Option<Person> {
    // Parse person id from url
    let person_id = match req
        .params
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
            formatcp!(
                "SELECT {} FROM persons WHERE id = ? LIMIT 1",
                Person::columns()
            ),
            person_id,
        )
        .next()
}

fn persons_show(req: &Request, ctx: &Context) -> Response {
    // Get person
    let person = match get_person(req, ctx) {
        Some(person) => person,
        None => return not_found(req, ctx),
    };

    // Return person
    Response::with_json(Into::<api::Person>::into(person))
}

fn persons_update(req: &Request, ctx: &Context) -> Response {
    // Get person
    let mut person = match get_person(req, ctx) {
        Some(person) => person,
        None => return not_found(req, ctx),
    };

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::PersonCreateUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<PersonCreateUpdateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Update person
    person.name = body.name;
    person.age_in_years = body.age_in_years;
    person.relation = body.relation;
    execute_args!(
        ctx.database,
        "UPDATE persons SET name = :name, age = :age, relation = :relation WHERE id = :id",
        Args {
            id: person.id,
            name: person.name.clone(),
            age: person.age_in_years,
            relation: person.relation
        }
    );

    // Return updated person
    Response::with_json(Into::<api::Person>::into(person))
}

fn persons_delete(req: &Request, ctx: &Context) -> Response {
    // Get person
    let person = match get_person(req, ctx) {
        Some(person) => person,
        None => return not_found(req, ctx),
    };

    // Delete person
    ctx.database
        .execute("DELETE FROM persons WHERE id = ?", person.id);

    // Success response
    Response::new()
}

// MARK: Main
fn router(ctx: Context) -> Router<Context> {
    RouterBuilder::<Context>::with(ctx)
        .pre_layer(layers::log_pre_layer)
        .pre_layer(layers::cors_pre_layer)
        .post_layer(layers::cors_post_layer)
        .get("/", home)
        .get("/persons", persons_index)
        .post("/persons", persons_create)
        .get("/persons/:person_id", persons_show)
        .put("/persons/:person_id", persons_update)
        .delete("/persons/:person_id", persons_delete)
        .fallback(not_found)
        .build()
}

fn main() {
    // Init logger
    simple_logger::init().expect("Failed to init logger");

    // Load environment variables
    _ = dotenv::dotenv();
    let http_port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(8080);

    // Load database
    let context = Context::with_database("database.db");

    // Start server
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, http_port))
        .unwrap_or_else(|_| panic!("Can't bind to port: {http_port}"));
    info!("Server is listening on: http://localhost:{http_port}/");

    let router = router(context);
    small_http::serve(listener, move |req| router.handle(req));
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_home() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::get("http://localhost/"));
        assert_eq!(res.status, Status::Ok);
        assert!(res.body.starts_with(b"Persons v"));
    }

    #[test]
    fn test_cors() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::get("http://localhost/"));
        assert_eq!(res.headers.get("Access-Control-Allow-Origin"), Some("*"));
    }

    #[test]
    fn test_cors_preflight() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::options("http://localhost/"));
        assert_eq!(res.headers.get("Access-Control-Allow-Origin"), Some("*"));
        assert_eq!(
            res.headers.get("Access-Control-Allow-Methods"),
            Some("GET, POST, PUT, DELETE")
        );
        assert_eq!(res.headers.get("Access-Control-Max-Age"), Some("86400"));
    }

    #[test]
    fn test_persons_index() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Fetch /persons check if empty
        let res = router.handle(&Request::get("http://localhost/persons"));
        assert_eq!(res.status, Status::Ok);
        let persons = serde_json::from_slice::<api::PersonIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert!(persons.is_empty());

        // Create person
        let person = Person {
            name: "Jan".to_string(),
            age_in_years: 40,
            relation: Relation::Me,
            ..Default::default()
        };
        ctx.database.insert_person(person.clone());

        // Fetch /persons check if person is there
        let res = router.handle(&Request::get("http://localhost/persons"));
        assert_eq!(res.status, Status::Ok);
        let persons = serde_json::from_slice::<api::PersonIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].name, "Jan");
    }

    #[test]
    fn test_persons_index_search() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create multiple persons
        ctx.database.insert_person(Person {
            name: "Alice".to_string(),
            ..Default::default()
        });
        ctx.database.insert_person(Person {
            name: "Bob".to_string(),
            ..Default::default()
        });

        // Search for "Alice"
        let res = router.handle(&Request::get("http://localhost/persons?q=Alice"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].name, "Alice");
    }

    #[test]
    fn test_persons_index_pagination() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create multiple persons
        for i in 1..=30 {
            ctx.database.insert_person(Person {
                name: format!("Person {i}"),
                age_in_years: 20 + i,
                relation: Relation::Me,
                ..Default::default()
            });
        }

        // Fetch /persons with limit 10 and page 1
        let res = router.handle(&Request::get("http://localhost/persons?limit=10&page=1"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 10);
        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.limit, 10);
        assert_eq!(response.pagination.total, 30);

        // Fetch /persons with limit 10 and page 2
        let res = router.handle(&Request::get("http://localhost/persons?limit=5&page=2"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 5);
        assert_eq!(response.pagination.page, 2);
        assert_eq!(response.pagination.limit, 5);
        assert_eq!(response.pagination.total, 30);
        assert_eq!(response.data[0].name, "Person 6");
    }

    #[test]
    fn test_persons_create() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create person
        let res = router.handle(
            &Request::post("http://localhost/persons").body("name=Jan&ageInYears=40&relation=me"),
        );
        assert_eq!(res.status, Status::Ok);
        let person = serde_json::from_slice::<api::Person>(&res.body).unwrap();
        assert_eq!(person.name, "Jan");
    }

    #[test]
    fn test_persons_show() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create person
        let person = Person {
            name: "Jan".to_string(),
            age_in_years: 40,
            relation: Relation::Me,
            ..Default::default()
        };
        ctx.database.insert_person(person.clone());

        // Fetch /persons/:person_id check if person is there
        let res = router.handle(&Request::get(format!(
            "http://localhost/persons/{}",
            person.id
        )));
        assert_eq!(res.status, Status::Ok);
        let person = serde_json::from_slice::<api::Person>(&res.body).unwrap();
        assert_eq!(person.name, "Jan");

        // Fetch other person by random id should be 404 Not Found
        let res = router.handle(&Request::get(format!(
            "http://localhost/persons/{}",
            Uuid::now_v7()
        )));
        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn test_persons_update() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create person
        let person = Person {
            name: "Jan".to_string(),
            age_in_years: 40,
            relation: Relation::Me,
            ..Default::default()
        };
        ctx.database.insert_person(person.clone());

        // Update person
        let res = router.handle(
            &Request::put(format!("http://localhost/persons/{}", person.id))
                .body("name=Jan&ageInYears=41&relation=me"),
        );
        assert_eq!(res.status, Status::Ok);
        let person = serde_json::from_slice::<api::Person>(&res.body).unwrap();
        assert_eq!(person.age_in_years, 41);

        // Update person with validation errors
        let res = router.handle(
            &Request::put(format!("http://localhost/persons/{}", person.id))
                .body("name=Bastiaan&ageInYears=41&relation=wrong"),
        );
        assert_eq!(res.status, Status::BadRequest);
    }

    #[test]
    fn test_persons_delete() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create person
        let person = Person {
            name: "Jan".to_string(),
            age_in_years: 40,
            relation: Relation::Me,
            ..Default::default()
        };
        ctx.database.insert_person(person.clone());

        // Delete person
        let res = router.handle(&Request::delete(format!(
            "http://localhost/persons/{}",
            person.id
        )));
        assert_eq!(res.status, Status::Ok);

        // Fetch /persons check if empty
        let res = router.handle(&Request::get("http://localhost/persons"));
        assert_eq!(res.status, Status::Ok);
        let persons = serde_json::from_slice::<api::PersonIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert!(persons.is_empty());
    }
}
