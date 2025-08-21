#include <stdlib.h>
#include <stdio.h>
#include <sqlite3.h>

int main(void) {
    // Open an in-memory SQLite database
    sqlite3 *db;
    if (sqlite3_open(":memory:", &db) != SQLITE_OK) {
        fprintf(stderr, "Cannot open database: %s\n", sqlite3_errmsg(db));
        return EXIT_FAILURE;
    }

    // Create table persons
    const char *sql_create = "CREATE TABLE persons (\
        id INTEGER PRIMARY KEY AUTOINCREMENT,\
        name TEXT NOT NULL,\
        age INTEGER NOT NULL\
    );";
    char *err_msg = NULL;
    if (sqlite3_exec(db, sql_create, 0, 0, &err_msg) != SQLITE_OK) {
        fprintf(stderr, "SQL error: %s\n", err_msg);
        return EXIT_FAILURE;
    }

    // Insert two rows
    const char *sql_insert = "INSERT INTO persons (name, age) VALUES ('Alice', 30), ('Bob', 25);";
    if (sqlite3_exec(db, sql_insert, 0, 0, &err_msg) != SQLITE_OK) {
        fprintf(stderr, "SQL error: %s\n", err_msg);
        return EXIT_FAILURE;
    }

    // Read rows back
    const char *sql_select = "SELECT id, name, age FROM persons;";
    sqlite3_stmt *stmt;
    if (sqlite3_prepare_v2(db, sql_select, -1, &stmt, 0) != SQLITE_OK) {
        fprintf(stderr, "Failed to fetch data: %s\n", sqlite3_errmsg(db));
        return EXIT_FAILURE;
    }

    printf("Persons:\n");
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        int id = sqlite3_column_int(stmt, 0);
        const unsigned char *name = sqlite3_column_text(stmt, 1);
        int age = sqlite3_column_int(stmt, 2);
        printf("- ID: %d, Name: %s, Age: %d\n", id, name, age);
    }

    sqlite3_finalize(stmt);
    sqlite3_close(db);
    return EXIT_SUCCESS;
}
