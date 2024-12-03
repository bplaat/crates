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

fn handler(req: &http::Request, res: &mut http::Response, ctx: Context) {
    println!("{} {}", req.method, req.path);

    if req.path == "/" {
        res.set_header("Content-Type", "text/html");
        res.body = String::from("<h1>Hello World!</h1>");
        return;
    }

    if req.path == "/greet" {
        if req.method == "POST" {
            println!("{}", req.body);
            if let Ok(body) = serde_urlencoded::from_str::<GreetBody>(req.body.as_str()) {
                res.status = 200;
                res.body = format!("Hello {}!", body.name);
                return;
            }
            res.status = 400;
            res.body = String::from("Bad Request");
        } else {
            res.status = 405;
            res.body = String::from("Method Not Allowed");
        }
        return;
    }

    if req.path == "/redirect" {
        res.status = 307;
        res.set_header("Location", "/");
        return;
    }

    if req.path == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        res.set_header("Content-Type", "text/html");
        res.body = String::from("<h1>Sleeping done!</h1>");
        return;
    }

    if req.path == "/persons" {
        res.set_header("Content-Type", "application/json");
        res.body = serde_json::to_string(&ctx.persons).unwrap();
        return;
    }

    if req.path.starts_with("/persons/") {
        let person_id = match req.path["/persons/".len()..].parse::<Uuid>() {
            Ok(id) => id,
            Err(_) => {
                res.status = 400;
                res.body = "400 Bad Request".into();
                return;
            }
        };

        let person = ctx.persons.iter().find(|p| p.id == person_id);
        match person {
            Some(p) => {
                res.set_header("Content-Type", "application/json");
                res.body = serde_json::to_string(p).unwrap();
            }
            None => {
                res.status = 404;
                res.body = "404 Not Found".to_string();
            }
        }
        return;
    }

    res.status = 404;
    res.set_header("Content-Type", "text/html");
    res.body = "<h1>404 Not Found</h1>".into();
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
    http::serve_with_context(handler, HTTP_PORT, ctx);
}
