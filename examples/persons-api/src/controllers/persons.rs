/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

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
pub(crate) fn persons_index(req: &Request, ctx: &Context) -> Response {
    // Parse request query
    let query = match req.url.query() {
        Some(query) => match serde_urlencoded::from_str::<IndexQuery>(query) {
            Ok(query) => query,
            Err(_) => return Response::with_status(Status::BadRequest),
        },
        None => IndexQuery::default(),
    };
    if let Err(report) = query.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
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

pub(crate) fn persons_create(req: &Request, ctx: &Context) -> Response {
    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::PersonCreateUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<PersonCreateUpdateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
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

// MARK: Persons Show
pub(crate) fn persons_show(req: &Request, ctx: &Context) -> Response {
    // Get person
    let person = match get_person(req, ctx) {
        Some(person) => person,
        None => return not_found(req, ctx),
    };

    // Return person
    Response::with_json(Into::<api::Person>::into(person))
}

// MARK: Persons Update
pub(crate) fn persons_update(req: &Request, ctx: &Context) -> Response {
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
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
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

// MARK: Persons Delete
pub(crate) fn persons_delete(req: &Request, ctx: &Context) -> Response {
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

// MARK: Helpers
fn get_person(req: &Request, ctx: &Context) -> Option<Person> {
    // Parse person id from url
    let person_id = match req
        .params
        .get("person_id")
        .expect("person_id param should be present")
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
