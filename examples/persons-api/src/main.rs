/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple persons REST API example

use std::net::{Ipv4Addr, TcpListener};

use chrono::{DateTime, Utc};
use http::{Method, Request, Response, Status};
use router::{Router, RouterBuilder};
use serde::Deserialize;
use sqlite::{FromRow, FromValue};
use uuid::Uuid;
use validate::Validate;

mod api {
    include!(concat!(env!("OUT_DIR"), "/persons_api.rs"));
}

// MARK: Consts
mod consts {
    pub(crate) const DATABASE_PATH: &str = "database.db";
    pub(crate) const HTTP_PORT: u16 = 8080;
    pub(crate) const LIMIT_DEFAULT: i64 = 20;
    pub(crate) const LIMIT_MAX: i64 = 50;
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
    database: sqlite::Connection,
}

impl Context {
    fn with_database(database_path: &str) -> Self {
        let database = database_open(database_path).expect("Can't open database");
        database_seed(&database);
        Self { database }
    }

    #[cfg(test)]
    fn with_test_database() -> Self {
        let database = database_open(":memory:").expect("Can't open database");
        Self { database }
    }
}

// MARK: Database
trait DatabaseHelpers {
    fn insert_person(&self, person: Person);
}
impl DatabaseHelpers for sqlite::Connection {
    fn insert_person(&self, person: Person) {
        self.execute(
            format!(
                "INSERT INTO persons ({}) VALUES ({})",
                Person::columns(),
                Person::values()
            ),
            person,
        );
    }
}

fn database_open(path: &str) -> Result<sqlite::Connection, sqlite::ConnectionError> {
    let database = sqlite::Connection::open(path)?;
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
    Ok(database)
}

fn database_seed(database: &sqlite::Connection) {
    // Insert persons
    let persons_count = database
        .query::<i64>("SELECT COUNT(id) FROM persons", ())
        .next()
        .expect("Should be some");
    if persons_count == 0 {
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

    pub(crate) fn log(req: &Request, _: &mut Context) -> Option<Response> {
        println!("{} {}", req.method, req.url.path);
        None
    }

    pub(crate) fn cors_pre(req: &Request, _: &mut Context) -> Option<Response> {
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

    pub(crate) fn cors_post(_: &Request, _: &mut Context, res: Response) -> Response {
        res.header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST")
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
fn home(_: &Request, _: &Context) -> Response {
    Response::with_body(concat!("Persons v", env!("CARGO_PKG_VERSION")))
}

fn not_found(_: &Request, _: &Context) -> Response {
    Response::with_status(Status::NotFound).body("404 Not found")
}

#[derive(Default, Deserialize, Validate)]
struct IndexQuery {
    #[serde(rename = "q")]
    query: Option<String>,
    #[validate(range(min = 1))]
    page: Option<i64>,
    #[validate(range(min = 1, max = consts::LIMIT_MAX))]
    limit: Option<i64>,
}

fn persons_index(req: &Request, ctx: &Context) -> Response {
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
    let search_query = format!("%{}%", query.query.unwrap_or_default().replace("%", "\\%"));
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(consts::LIMIT_DEFAULT);
    let total = ctx
        .database
        .query::<i64>(
            "SELECT COUNT(id) FROM persons WHERE name LIKE ?",
            search_query.clone(),
        )
        .next()
        .expect("Should be some");
    let persons = ctx
        .database
        .query::<Person>(
            format!(
                "SELECT {} FROM persons WHERE name LIKE ? LIMIT ? OFFSET ?",
                Person::columns()
            ),
            (search_query, limit, (page - 1) * limit),
        )
        .map(Into::<api::Person>::into)
        .collect::<Vec<_>>();

    // Return persons
    Response::with_json(api::PersonIndexResponse {
        pagination: api::Pagination { page, limit, total },
        data: persons,
    })
}

#[derive(Deserialize, Validate)]
struct PersonCreateUpdateBody {
    #[validate(ascii, length(min = 3, max = 25), custom(validators::name))]
    name: String,
    #[validate(range(min = 8))]
    age_in_years: i64,
    relation: api::Relation,
}

fn persons_create(req: &Request, ctx: &Context) -> Response {
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
            format!(
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
        .pre_layer(layers::log)
        .pre_layer(layers::cors_pre)
        .post_layer(layers::cors_post)
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
    let router = router(Context::with_database(consts::DATABASE_PATH));
    println!(
        "Server is listening on: http://localhost:{}/",
        consts::HTTP_PORT
    );
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, consts::HTTP_PORT))
        .unwrap_or_else(|_| panic!("Can't bind to port: {}", consts::HTTP_PORT));
    http::serve(listener, move |req| router.handle(req));
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_home() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::with_url("http://localhost/"));
        assert_eq!(res.status, Status::Ok);
        assert!(res.body.starts_with("Persons v"));
    }

    #[test]
    fn test_cors() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::with_url("http://localhost/"));
        assert_eq!(
            res.headers.get("Access-Control-Allow-Origin"),
            Some(&"*".to_string())
        );
    }

    #[test]
    fn test_persons_index() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Fetch /persons check if empty
        let res = router.handle(&Request::with_url("http://localhost/persons"));
        assert_eq!(res.status, Status::Ok);
        let persons = serde_json::from_str::<api::PersonIndexResponse>(&res.body)
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
        let res = router.handle(&Request::with_url("http://localhost/persons"));
        assert_eq!(res.status, Status::Ok);
        let persons = serde_json::from_str::<api::PersonIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].name, "Jan");
    }

    #[test]
    fn test_persons_create() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create person
        let res = router.handle(
            &Request::with_url("http://localhost/persons")
                .method(Method::Post)
                .body("name=Jan&age_in_years=40&relation=me"),
        );
        assert_eq!(res.status, Status::Ok);
        let person = serde_json::from_str::<api::Person>(&res.body).unwrap();
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
        let res = router.handle(&Request::with_url(format!(
            "http://localhost/persons/{}",
            person.id
        )));
        assert_eq!(res.status, Status::Ok);
        let person = serde_json::from_str::<api::Person>(&res.body).unwrap();
        assert_eq!(person.name, "Jan");

        // Fetch other person by random id should be 404 Not Found
        let res = router.handle(&Request::with_url(format!(
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
            &Request::with_url(format!("http://localhost/persons/{}", person.id))
                .method(Method::Put)
                .body("name=Jan&age_in_years=41&relation=me"),
        );
        assert_eq!(res.status, Status::Ok);
        let person = serde_json::from_str::<api::Person>(&res.body).unwrap();
        assert_eq!(person.age_in_years, 41);
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
        let res = router.handle(
            &Request::with_url(format!("http://localhost/persons/{}", person.id))
                .method(Method::Delete),
        );
        assert_eq!(res.status, Status::Ok);

        // Fetch /persons check if empty
        let res = router.handle(&Request::with_url("http://localhost/persons"));
        assert_eq!(res.status, Status::Ok);
        let persons = serde_json::from_str::<api::PersonIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert!(persons.is_empty());
    }
}