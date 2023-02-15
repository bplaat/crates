use crate::thread_pool::ThreadPool;
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    str,
};

pub struct Request {
    protocol: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    fn from_stream(stream: &mut TcpStream) -> Option<Request> {
        let mut reader = BufReader::new(stream);

        let mut line = String::new();
        _ = reader.read_line(&mut line);
        let mut req = {
            let mut parts = line.split(" ");
            let method = parts.next().unwrap().trim().to_string();
            let path = parts.next().unwrap().trim().to_string();
            let protocol = parts.next().unwrap().trim().to_string();
            Request {
                protocol,
                method,
                path,
                headers: HashMap::new(),
                body: String::new(),
            }
        };

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(_) => {
                    if line == "\r\n" {
                        break;
                    }
                    let mut parts = line.split(":");
                    req.headers.insert(
                        parts.next().unwrap().trim().to_string(),
                        parts.next().unwrap().trim().to_string(),
                    );
                }
                Err(_) => break,
            }
        }

        if req.method == "POST" {
            let length: usize = req.headers["Content-Length"].parse().unwrap();
            let mut buffer = vec![0_u8; length];
            _ = reader.read(&mut buffer);
            if let Ok(text) = str::from_utf8(&buffer) {
                req.body.push_str(text);
            }
        }
        Some(req)
    }
}

pub struct Response {
    protocol: String,
    pub status: i32,
    headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    fn write_to_stream(&self, stream: &mut TcpStream) {
        let mut sb = self.protocol.clone();
        sb.push(' ');
        sb.push_str(self.status.to_string().as_str());
        sb.push(' ');
        if self.status == 200 {
            sb.push_str("OK\r\n");
        }
        if self.status == 307 {
            sb.push_str("Temporary Redirect\r\n");
        }
        if self.status == 400 {
            sb.push_str("Bad Request\r\n");
        }
        if self.status == 404 {
            sb.push_str("Not Found\r\n");
        }
        if self.status == 405 {
            sb.push_str("Method Not Allowed\r\n");
        }

        for (name, value) in &self.headers {
            sb.push_str(name.as_str());
            sb.push_str(": ");
            sb.push_str(&value.as_str());
            sb.push_str("\r\n");
        }
        if self.protocol != "HTTP/1.0" {
            sb.push_str("Connection: close\r\n");
        }
        sb.push_str("Content-Length: ");
        sb.push_str(self.body.len().to_string().as_str());
        sb.push_str("\r\n\r\n");

        sb.push_str(self.body.as_str());
        _ = stream.write_all(sb.as_bytes());
    }
}

pub fn serve(callback: fn(&Request, &mut Response), port: i32) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    let pool = ThreadPool::new(16);
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            pool.execute(move || {
                if let Some(request) = Request::from_stream(&mut stream) {
                    let mut response = Response {
                        protocol: request.protocol.clone(),
                        status: 200,
                        headers: HashMap::new(),
                        body: String::new(),
                    };
                    callback(&request, &mut response);
                    response.write_to_stream(&mut stream);
                }
            });
        }
    }
}
