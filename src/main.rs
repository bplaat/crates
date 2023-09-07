use crate::uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::{thread, time::Duration};

mod http;
mod thread_pool;
mod uuid;

#[derive(Serialize)]
struct Person {
    id: Uuid,
    name: String,
    age: i32,
}

#[derive(Deserialize)]
struct GreetBody {
    name: String,
}

fn handler(req: &http::Request, res: &mut http::Response) {
    println!("{} {}", req.method, req.path);

    if req.path == "/" {
        res.set_header("Content-Type", "text/html");
        res.body = String::from("<h1>Hello World!</h1>");
        return;
    }

    if req.path == "/greet" {
        if req.method == "POST" {
            println!("{}", req.body);
            if let Ok(body) = serde_json::from_str::<GreetBody>(req.body.as_str()) {
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
        let mut persons = Vec::with_capacity(4);
        persons.push(Person {
            id: Uuid::new(),
            name: "Bastiaan".to_string(),
            age: 20,
        });
        persons.push(Person {
            id: Uuid::new(),
            name: "Sander".to_string(),
            age: 19,
        });
        persons.push(Person {
            id: Uuid::new(),
            name: "Leonard".to_string(),
            age: 16,
        });
        persons.push(Person {
            id: Uuid::new(),
            name: "Jiska".to_string(),
            age: 14,
        });

        res.set_header("Content-Type", "application/json");
        res.body = serde_json::to_string(&persons).unwrap();
        return;
    }

    res.status = 404;
    res.set_header("Content-Type", "text/html");
    res.body = String::from("<h1>404 Not Found</h1>");
}

fn main() {
    println!("Server is listening on http://localhost:8080/");
    http::serve(handler, 8080);
}
