/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::{Request, Response, Status};

mod http;

#[derive(Serialize, Clone)]
struct Person {
    id: Uuid,
    name: String,
    age: i32,
    created_at: DateTime<Utc>,
}

#[derive(Clone)]
struct Context {
    persons: Vec<Person>,
}

#[derive(Deserialize)]
struct GreetBody {
    name: String,
}

const HTTP_PORT: u16 = 8081;

fn handler(req: Request, ctx: Context) -> Response {
    println!("{} {}", req.method, req.path);

    if req.path == "/" {
        return Response::new().html("<h1>Hello World!</h1>");
    }

    if req.path == "/greet" {
        if req.method == "POST" {
            let body = match serde_urlencoded::from_str::<GreetBody>(&req.body) {
                Ok(body) => body,
                Err(_) => {
                    return Response::new()
                        .status(Status::BadRequest)
                        .body("Bad Request");
                }
            };
            return Response::new().html(format!("<h1>Hello {}!</h1>", body.name));
        }
        return Response::new()
            .status(Status::MethodNotAllowed)
            .body("Method Not Allowed");
    }

    if req.path == "/redirect" {
        return Response::new().redirect("/");
    }

    if req.path == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        return Response::new().html("<h1>Sleeping done!</h1>");
    }

    // REST API example
    if req.path.starts_with("/persons") {
        let res = Response::new().header("Access-Control-Allow-Origin", "*");

        if req.path == "/persons" {
            return res.json(&ctx.persons);
        }

        if req.path.starts_with("/persons/") {
            let person_id = match req.path["/persons/".len()..].parse::<Uuid>() {
                Ok(id) => id,
                Err(_) => {
                    return res.status(Status::BadRequest).body("Bad Request");
                }
            };

            if let Some(person) = ctx.persons.iter().find(|p| p.id == person_id) {
                return res.json(person);
            }
        }

        return res.status(Status::NotFound).body("Not Found");
    }

    Response::new()
        .status(Status::NotFound)
        .html("<h1>404 Not Found</h1>")
}

fn main() {
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
    let ctx = Context { persons };
    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    http::serve_with_ctx(handler, HTTP_PORT, ctx);
}
