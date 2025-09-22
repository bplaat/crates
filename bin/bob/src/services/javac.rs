/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::utils::spawn_service;

pub(crate) const JAVAC_SERVER_SOCKET: &str = "/tmp/.bob/javac";

const JAVAC_SERVER_CLASS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/JavacServer.class"));

pub(crate) fn start_javac_server() {
    // Check if server is already running
    if Path::new(JAVAC_SERVER_SOCKET).exists() {
        return;
    }

    let dir = Path::new(JAVAC_SERVER_SOCKET)
        .parent()
        .expect("Failed to get parent directory");

    // Create directory if it doesn't exist
    if !Path::new(dir).exists() {
        fs::create_dir_all(dir).expect("Failed to create /tmp/.bob directory");
    }

    // Write the JAVA_SERVER_CLASS bytes to the class file
    let class_path = dir.join("JavacServer.class");
    let mut file = File::create(&class_path).expect("Failed to create JavacServer.class");
    file.write_all(JAVAC_SERVER_CLASS)
        .expect("Failed to write JavacServer.class");

    // Set permissions to be readable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&class_path, fs::Permissions::from_mode(0o644))
            .expect("Failed to set permissions");
    }

    // Start the server
    spawn_service("java", &["-cp", &dir.display().to_string(), "JavacServer"])
        .expect("Can't start javac server");

    // Wait until the socket file exists
    println!("Starting javac server...");
    while !Path::new(JAVAC_SERVER_SOCKET).exists() {
        thread::sleep(Duration::from_millis(100));
    }
}
