use crate::uuid::Uuid;
use json::{JsonValue, ToJson};
use std::{thread, time::Duration};

mod http;
mod json;
mod thread_pool;
mod uuid;

struct Person {
    id: Uuid,
    name: String,
    age: i32,
}

impl ToJson for Person {
    fn to_json(&self) -> JsonValue {
        let mut person_json = JsonValue::new_object();
        person_json.insert("id", JsonValue::String(self.id.to_string()));
        person_json.insert("name", JsonValue::String(self.name.clone()));
        person_json.insert("age", JsonValue::Int(self.age));
        person_json
    }
}

fn handler(req: &http::Request, res: &mut http::Response) {
    println!("{} {}", req.method, req.path);

    if req.path == "/" {
        res.set_header("Content-Type", "text/html");
        res.body = String::from("<h1>Hello World!</h1>");
        return;
    }

    if req.path == "/login" {
        if req.method == "POST" {
            println!("{}", req.body);
            if let Some(body) = JsonValue::parse(req.body.as_str()) {
                if let JsonValue::Object(object) = body {
                    if let Some(JsonValue::String(name)) = object.get("name") {
                        res.status = 200;
                        res.body = format!("Hello {}!", name);
                        return;
                    }
                }
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
        res.body = persons.to_json().to_string();
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
