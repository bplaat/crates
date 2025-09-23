/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, thread};

use crate::utils::spawn_service;

const JAVAC_SERVER_CLASS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/JavacServer.class"));

pub(crate) fn javac_server_socket() -> PathBuf {
    env::temp_dir().join("bob").join("javac")
}

pub(crate) fn start_javac_server() {
    // Check if server is already running
    let javac_server_socket = javac_server_socket();
    if Path::new(&javac_server_socket).exists() {
        return;
    }

    // Create directory if it doesn't exist
    let parent_dir = javac_server_socket
        .parent()
        .expect("Failed to get parent directory");
    if !Path::new(parent_dir).exists() {
        fs::create_dir_all(parent_dir).expect("Failed to create dir");
    }

    // Write the JAVA_SERVER_CLASS bytes to the class file
    let class_path = parent_dir.join("JavacServer.class");
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
    spawn_service(
        "java",
        &["-cp", &parent_dir.display().to_string(), "JavacServer"],
    )
    .expect("Can't start javac server");

    // Wait until the socket file exists
    println!("Starting javac server...");
    while !javac_server_socket.exists() {
        thread::sleep(Duration::from_millis(100));
    }
}
