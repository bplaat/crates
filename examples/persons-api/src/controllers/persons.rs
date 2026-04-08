/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use bsqlite::{execute_args, query_args};
use const_format::formatcp;
use from_derive::FromStruct;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;

use crate::context::{Context, DatabaseHelpers};
use crate::controllers::not_found;
use crate::models::{IndexQuery, Person, Relation};
use crate::{api, validators};

// MARK: Persons Index
pub(crate) fn persons_index(req: &Request, ctx: &Context) -> Result<Response> {
    // Parse request query
    let query = match req.url.query() {
        Some(query) => match serde_urlencoded::from_str::<IndexQuery>(query) {
            Ok(query) => query,
            Err(_) => return Ok(Response::with_status(Status::BadRequest)),
        },
        None => IndexQuery::default(),
    };
    if let Err(report) = query.validate() {
        return Ok(Response::with_status(Status::BadRequest).json(api::Report::from(report)));
    }

    // Get persons
    let (total, persons) = if query.query.is_empty() {
        let total = ctx
            .database
            .query_some::<i64>("SELECT COUNT(id) FROM persons", ())?;
        let persons = query_args!(
            Person,
            ctx.database,
            formatcp!(
                "SELECT {} FROM persons ORDER BY created_at DESC LIMIT :limit OFFSET :offset",
                Person::columns()
            ),
            Args {
                limit: query.limit,
                offset: (query.page - 1) * query.limit
            }
        )?
        .map(|r| r.map(Into::into).map_err(anyhow::Error::from))
        .collect::<Result<Vec<api::Person>>>()?;
        (total, persons)
    } else {
        let total = ctx.database.query_some::<i64>(
            "SELECT COUNT(rowid) FROM persons_fts WHERE persons_fts MATCH ?",
            query.query.clone(),
        )?;
        let persons = query_args!(
            Person,
            ctx.database,
            "SELECT p.id, p.name, p.age, p.relation, p.created_at FROM persons p
            JOIN persons_fts fts ON p.id = fts.id
            WHERE persons_fts MATCH :fts_query
            ORDER BY p.created_at DESC LIMIT :limit OFFSET :offset",
            Args {
                fts_query: query.query,
                limit: query.limit,
                offset: (query.page - 1) * query.limit
            }
        )?
        .map(|r| r.map(Into::into).map_err(anyhow::Error::from))
        .collect::<Result<Vec<api::Person>>>()?;
        (total, persons)
    };

    // Return persons
    Ok(Response::with_json(api::PersonIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: persons,
    }))
}

// MARK: Persons Create
#[derive(Validate, FromStruct)]
#[from_struct(api::PersonCreateUpdateBody)]
struct PersonCreateUpdateBody {
    #[validate(ascii, length(min = 3, max = 25), custom(validators::name_validator))]
    name: String,
    #[validate(range(min = 8))]
    age_in_years: i64,
    relation: Relation,
}

pub(crate) fn persons_create(req: &Request, ctx: &Context) -> Result<Response> {
    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::PersonCreateUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => PersonCreateUpdateBody::from(body),
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    if let Err(report) = body.validate() {
        return Ok(Response::with_status(Status::BadRequest).json(api::Report::from(report)));
    }

    // Create person
    let person = Person {
        name: body.name,
        age_in_years: body.age_in_years,
        relation: body.relation,
        ..Default::default()
    };
    ctx.database.insert_person(person.clone())?;

    // Return created person
    Ok(Response::with_json(api::Person::from(person)))
}

// MARK: Persons Show
pub(crate) fn persons_show(req: &Request, ctx: &Context) -> Result<Response> {
    // Get person
    let person_id = match parse_person_id(req) {
        Ok(id) => id,
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    let person = match get_person(person_id, ctx)? {
        Some(person) => person,
        None => return not_found(req, ctx),
    };

    // Return person
    Ok(Response::with_json(api::Person::from(person)))
}

// MARK: Persons Update
pub(crate) fn persons_update(req: &Request, ctx: &Context) -> Result<Response> {
    // Get person
    let person_id = match parse_person_id(req) {
        Ok(id) => id,
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    let mut person = match get_person(person_id, ctx)? {
        Some(person) => person,
        None => return not_found(req, ctx),
    };

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::PersonCreateUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => PersonCreateUpdateBody::from(body),
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    if let Err(report) = body.validate() {
        return Ok(Response::with_status(Status::BadRequest).json(api::Report::from(report)));
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
    )?;

    // Return updated person
    Ok(Response::with_json(api::Person::from(person)))
}

// MARK: Persons Delete
pub(crate) fn persons_delete(req: &Request, ctx: &Context) -> Result<Response> {
    // Get person
    let person_id = match parse_person_id(req) {
        Ok(id) => id,
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    let person = match get_person(person_id, ctx)? {
        Some(person) => person,
        None => return not_found(req, ctx),
    };

    // Delete person
    ctx.database
        .execute("DELETE FROM persons WHERE id = ?", person.id)?;

    // Success response
    Ok(Response::with_status(Status::NoContent))
}

// MARK: Helpers
fn parse_person_id(req: &Request) -> Result<Uuid> {
    match req
        .params
        .get("person_id")
        .and_then(|id| id.parse::<Uuid>().ok())
    {
        Some(id) => Ok(id),
        None => anyhow::bail!("Invalid UUID"),
    }
}

fn get_person(person_id: Uuid, ctx: &Context) -> Result<Option<Person>> {
    Ok(ctx
        .database
        .query::<Person>(
            formatcp!(
                "SELECT {} FROM persons WHERE id = ? LIMIT 1",
                Person::columns()
            ),
            person_id,
        )?
        .next()
        .transpose()
        .map_err(Box::new)?)
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::router;

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
        ctx.database.insert_person(person.clone()).unwrap();

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
        ctx.database
            .insert_person(Person {
                name: "Alice".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_person(Person {
                name: "Bob".to_string(),
                ..Default::default()
            })
            .unwrap();

        // Search for "Alice"
        let res = router.handle(&Request::get("http://localhost/persons?q=Alice"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].name, "Alice");
    }

    #[test]
    fn test_persons_index_fts5_search() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        ctx.database
            .insert_person(Person {
                name: "Alice Smith".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_person(Person {
                name: "Alice Johnson".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_person(Person {
                name: "Bob Smith".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_person(Person {
                name: "Carol White".to_string(),
                ..Default::default()
            })
            .unwrap();

        // Prefix search
        let res = router.handle(&Request::get("http://localhost/persons?q=Al*"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 2);

        // AND search
        let res = router.handle(&Request::get("http://localhost/persons?q=Alice AND Smith"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].name, "Alice Smith");

        // OR search
        let res = router.handle(&Request::get("http://localhost/persons?q=Alice OR Bob"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 3);

        // NOT search
        let res = router.handle(&Request::get("http://localhost/persons?q=Alice NOT Smith"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].name, "Alice Johnson");

        // Phrase search
        let res = router.handle(&Request::get("http://localhost/persons?q=\"Alice Smith\""));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::PersonIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].name, "Alice Smith");
    }

    #[test]
    fn test_persons_index_pagination() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create multiple persons
        for i in 1..=30 {
            ctx.database
                .insert_person(Person {
                    name: format!("Person {i}"),
                    age_in_years: 20 + i,
                    relation: Relation::Me,
                    ..Default::default()
                })
                .unwrap();
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
        ctx.database.insert_person(person.clone()).unwrap();

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
        ctx.database.insert_person(person.clone()).unwrap();

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
        ctx.database.insert_person(person.clone()).unwrap();

        // Delete person
        let res = router.handle(&Request::delete(format!(
            "http://localhost/persons/{}",
            person.id
        )));
        assert_eq!(res.status, Status::NoContent);

        // Fetch /persons check if empty
        let res = router.handle(&Request::get("http://localhost/persons"));
        assert_eq!(res.status, Status::Ok);
        let persons = serde_json::from_slice::<api::PersonIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert!(persons.is_empty());
    }
}
