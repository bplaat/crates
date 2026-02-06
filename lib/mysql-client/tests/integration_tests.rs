/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Integration tests for mysql-client.

use mysql_client::{Connection, Value};

const HOST: &str = "127.0.0.1";
const PORT: u16 = 3306;
const USER: &str = "test";
const PASSWORD: &str = "test";
const DATABASE: &str = "test";

fn setup_connection() -> mysql_client::error::Result<Connection> {
    Connection::connect(HOST, PORT, USER, PASSWORD, DATABASE)
}

fn setup_test_table(conn: &mut Connection) -> mysql_client::error::Result<()> {
    let _ = conn.query("DROP TABLE IF EXISTS test_users");
    conn.query("CREATE TABLE test_users (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(100), age INT, email VARCHAR(100))")?;
    Ok(())
}

#[test]
fn test_connection_succeeds() {
    match setup_connection() {
        Ok(_) => {},
        Err(e) => panic!("Failed to connect: {e}"),
    }
}

#[test]
fn test_create_table() {
    let mut conn = setup_connection().expect("Failed to connect");
    let result = setup_test_table(&mut conn);
    assert!(result.is_ok(), "Should create test table");
}

#[test]
fn test_insert_query() {
    let mut conn = setup_connection().expect("Failed to connect");
    setup_test_table(&mut conn).expect("Failed to setup");

    let result = conn.query(
        "INSERT INTO test_users (name, age, email) VALUES ('Alice', 30, 'alice@example.com')",
    );
    assert!(result.is_ok(), "INSERT should succeed");
}

#[test]
fn test_select_empty_table() {
    let mut conn = setup_connection().expect("Failed to connect");
    setup_test_table(&mut conn).expect("Failed to setup");

    let rows = conn
        .query("SELECT id, name, age, email FROM test_users")
        .expect("SELECT should succeed");
    assert_eq!(rows.len(), 0, "Empty table should return 0 rows");
}

#[test]
fn test_insert_and_select() {
    let mut conn = setup_connection().expect("Failed to connect");
    setup_test_table(&mut conn).expect("Failed to setup");

    conn.query("INSERT INTO test_users (name, age, email) VALUES ('Bob', 25, 'bob@example.com')")
        .expect("INSERT should succeed");
    conn.query(
        "INSERT INTO test_users (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')",
    )
    .expect("INSERT should succeed");

    let rows = conn
        .query("SELECT id, name, age, email FROM test_users ORDER BY name")
        .expect("SELECT should succeed");
    assert_eq!(rows.len(), 2, "Should have 2 rows");
    assert_eq!(
        rows[0][1],
        Value::Bytes("Bob".as_bytes().to_vec()),
        "First name should be Bob"
    );
    assert_eq!(
        rows[1][1],
        Value::Bytes("Charlie".as_bytes().to_vec()),
        "Second name should be Charlie"
    );
}

#[test]
fn test_update_query() {
    let mut conn = setup_connection().expect("Failed to connect");
    setup_test_table(&mut conn).expect("Failed to setup");

    conn.query(
        "INSERT INTO test_users (name, age, email) VALUES ('David', 40, 'david@example.com')",
    )
    .expect("INSERT should succeed");

    let result = conn.query("UPDATE test_users SET age = 41 WHERE name = 'David'");
    assert!(result.is_ok(), "UPDATE should succeed");

    let rows = conn
        .query("SELECT age FROM test_users WHERE name = 'David'")
        .expect("SELECT should succeed");
    assert_eq!(rows.len(), 1, "Should find David");
}

#[test]
fn test_delete_query() {
    let mut conn = setup_connection().expect("Failed to connect");
    setup_test_table(&mut conn).expect("Failed to setup");

    conn.query("INSERT INTO test_users (name, age, email) VALUES ('Eve', 28, 'eve@example.com')")
        .expect("INSERT should succeed");
    conn.query(
        "INSERT INTO test_users (name, age, email) VALUES ('Frank', 32, 'frank@example.com')",
    )
    .expect("INSERT should succeed");

    let result = conn.query("DELETE FROM test_users WHERE name = 'Eve'");
    assert!(result.is_ok(), "DELETE should succeed");

    let rows = conn
        .query("SELECT * FROM test_users")
        .expect("SELECT should succeed");
    assert_eq!(rows.len(), 1, "Should have 1 row after delete");
}

#[test]
fn test_value_as_string() {
    let value = Value::Bytes("hello".as_bytes().to_vec());
    let string = value.as_string().expect("Should convert to string");
    assert_eq!(string, "hello");
}

#[test]
fn test_value_as_int() {
    let value = Value::Int(42);
    let int = value.as_int().expect("Should convert to int");
    assert_eq!(int, 42);
}

#[test]
fn test_value_as_uint() {
    let value = Value::UInt(100);
    let uint = value.as_uint().expect("Should convert to uint");
    assert_eq!(uint, 100);
}

#[test]
fn test_value_as_float() {
    let value = Value::Float(3.14);
    let float = value.as_float().expect("Should convert to float");
    assert!((float - 3.14).abs() < 0.01);
}

#[test]
fn test_value_is_null() {
    let null_value = Value::Null;
    assert!(null_value.is_null());

    let non_null = Value::Int(5);
    assert!(!non_null.is_null());
}

#[test]
fn test_value_null_conversion_error() {
    let null_value = Value::Null;
    let result = null_value.as_string();
    assert!(result.is_err(), "Converting NULL to string should fail");
}

#[test]
fn test_multiple_rows_with_different_types() {
    let mut conn = setup_connection().expect("Failed to connect");
    setup_test_table(&mut conn).expect("Failed to setup");

    conn.query("INSERT INTO test_users (name, age, email) VALUES ('User1', 20, 'user1@test.com')")
        .expect("INSERT should succeed");
    conn.query("INSERT INTO test_users (name, age, email) VALUES ('User2', 30, 'user2@test.com')")
        .expect("INSERT should succeed");
    conn.query("INSERT INTO test_users (name, age, email) VALUES ('User3', 40, 'user3@test.com')")
        .expect("INSERT should succeed");

    let rows = conn
        .query("SELECT id, name, age, email FROM test_users ORDER BY id")
        .expect("SELECT should succeed");
    assert_eq!(rows.len(), 3, "Should have 3 rows");

    // Verify structure of first row
    assert_eq!(rows[0].len(), 4, "Each row should have 4 columns");
    assert!(!rows[0][0].is_null(), "ID should not be null");
    assert_eq!(rows[0][1], Value::Bytes("User1".as_bytes().to_vec()));
    assert_eq!(
        rows[0][3],
        Value::Bytes("user1@test.com".as_bytes().to_vec())
    );
}

#[test]
fn test_null_in_results() {
    let mut conn = setup_connection().expect("Failed to connect");
    let _ = conn.query("DROP TABLE IF EXISTS test_nullable");
    conn.query("CREATE TABLE test_nullable (id INT PRIMARY KEY, optional_field VARCHAR(100))")
        .expect("CREATE TABLE should succeed");

    conn.query("INSERT INTO test_nullable (id, optional_field) VALUES (1, 'value')")
        .expect("INSERT should succeed");
    conn.query("INSERT INTO test_nullable (id, optional_field) VALUES (2, NULL)")
        .expect("INSERT should succeed");

    let rows = conn
        .query("SELECT id, optional_field FROM test_nullable ORDER BY id")
        .expect("SELECT should succeed");
    assert_eq!(rows.len(), 2, "Should have 2 rows");
    assert!(!rows[0][1].is_null(), "First row field should not be null");
    assert!(rows[1][1].is_null(), "Second row field should be null");

    let _ = conn.query("DROP TABLE test_nullable");
}

#[test]
fn test_select_with_where_clause() {
    let mut conn = setup_connection().expect("Failed to connect");
    setup_test_table(&mut conn).expect("Failed to setup");

    conn.query("INSERT INTO test_users (name, age, email) VALUES ('Alice', 25, 'alice@test.com')")
        .expect("INSERT should succeed");
    conn.query("INSERT INTO test_users (name, age, email) VALUES ('Bob', 35, 'bob@test.com')")
        .expect("INSERT should succeed");
    conn.query(
        "INSERT INTO test_users (name, age, email) VALUES ('Charlie', 25, 'charlie@test.com')",
    )
    .expect("INSERT should succeed");

    let rows = conn
        .query("SELECT name FROM test_users WHERE age = 25 ORDER BY name")
        .expect("SELECT should succeed");
    assert_eq!(rows.len(), 2, "Should find 2 users with age 25");
}

#[test]
fn test_count_query() {
    let mut conn = setup_connection().expect("Failed to connect");
    setup_test_table(&mut conn).expect("Failed to setup");

    conn.query("INSERT INTO test_users (name, age, email) VALUES ('User1', 20, 'u1@test.com')")
        .expect("INSERT should succeed");
    conn.query("INSERT INTO test_users (name, age, email) VALUES ('User2', 20, 'u2@test.com')")
        .expect("INSERT should succeed");

    let rows = conn
        .query("SELECT COUNT(*) FROM test_users")
        .expect("SELECT should succeed");
    assert_eq!(rows.len(), 1, "COUNT should return 1 row");
}
