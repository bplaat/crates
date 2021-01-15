// PlaatCraft - Database

#include "database.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "config.h"
#include "utils.h"

Database* database_new(void) {
    Database* database = malloc(sizeof(Database));

    // Init database connection
    if (sqlite3_open("assets/world.db", &database->database) != SQLITE_OK) {
        log_error("Can't open the SQLite database:\n%s", sqlite3_errmsg(database->database));
    }
    mtx_init(&database->database_lock, mtx_plain);
    database->database_changes = 0;

    // Create settings table if not exists
    char *error_message = NULL;
    if (sqlite3_exec(database->database, "CREATE TABLE IF NOT EXISTS [settings] ("
        "[key] VARCHAR(32) UNIQUE NOT NULL,"
        "[value] VARCHAR(255) NOT NULL"
    ")", NULL, NULL, &error_message) != SQLITE_OK) {
        log_error("Can't create the settings table:\n%s", error_message);
    }

    // Init settings select statement
    if (sqlite3_prepare_v2(database->database, "SELECT [value] FROM [settings] WHERE [key] = ?", -1, &database->settings_select_statement, NULL) != SQLITE_OK) {
        log_error("Can't create settings select statement");
    }

    // Init settings insert statement
    if (sqlite3_prepare_v2(database->database, "INSERT INTO [settings] ([key], [value]) VALUES (?, ?)", -1, &database->settings_insert_statement, NULL) != SQLITE_OK) {
        log_error("Can't create settings insert statement");
    }

    // Init settings update statement
    if (sqlite3_prepare_v2(database->database, "UPDATE [settings] SET [value] = ? WHERE [key] = ?", -1, &database->settings_update_statement, NULL) != SQLITE_OK) {
        log_error("Can't create settings update statement");
    }

    // Create chunks table if not exists
    if (sqlite3_exec(database->database, "CREATE TABLE IF NOT EXISTS [chunks] ("
        "[x] INT NOT NULL,"
        "[y] INT NOT NULL,"
        "[z] INT NOT NULL,"
        "[data] BLOB NOT NULL,"
        "UNIQUE([x], [y], [z])"
    ")", NULL, NULL, &error_message) != SQLITE_OK) {
        log_error("Can't create the chunks table:\n%s", error_message);
    }

    // Init chunks select statement
    if (sqlite3_prepare_v2(database->database, "SELECT [data] FROM [chunks] WHERE [x] = ? AND [y] = ? AND [z] = ?", -1, &database->chunks_select_statement, NULL) != SQLITE_OK) {
        log_error("Can't create chunks select statement");
    }

    // Init chunks insert statement
    if (sqlite3_prepare_v2(database->database, "INSERT INTO [chunks] ([x], [y], [z], [data]) VALUES (?, ?, ?, ?)", -1, &database->chunks_insert_statement, NULL) != SQLITE_OK) {
        log_error("Can't create chunks insert statement");
    }

    // Init chunks update statement
    if (sqlite3_prepare_v2(database->database, "UPDATE [chunks] SET [data] = ? WHERE [x] = ? AND [y] = ? AND [z] = ?", -1, &database->chunks_update_statement, NULL) != SQLITE_OK) {
        log_error("Can't create chunks update statement");
    }

    // Begin database transaction
    if (sqlite3_exec(database->database, "BEGIN", NULL, NULL, &error_message) != SQLITE_OK) {
        log_error("Can't begin database transaction:\n%s", error_message);
    }

    return database;
}

char* database_settings_get_string(Database* database, char* key, char* default_value) {
    mtx_lock(&database->database_lock);

    char* value;

    // Do a select setting query to find the setting
    sqlite3_reset(database->settings_select_statement);
    sqlite3_bind_text(database->settings_select_statement, 1, key, strlen(key), SQLITE_STATIC);
    if (sqlite3_step(database->settings_select_statement) == SQLITE_ROW) {
        const uint8_t* value_string = sqlite3_column_text(database->settings_select_statement, 0);
        value = string_copy((char*)value_string);
    } else {
        value = default_value;
    }

    mtx_unlock(&database->database_lock);

    return value;
}

int database_settings_get_int(Database* database, char* key, int default_value) {
    char *value_string = database_settings_get_string(database, key, NULL);
    if (value_string != NULL) {
        int value = atoi(value_string);
        free(value_string);
        return value;
    } else {
        return default_value;
    }
}

float database_settings_get_float(Database* database, char* key, float default_value) {
    char *value_string = database_settings_get_string(database, key, NULL);
    if (value_string != NULL) {
        float value = atof(value_string);
        free(value_string);
        return value;
    } else {
        return default_value;
    }
}

void database_settings_set_string(Database* database, char* key, char* value) {
    mtx_lock(&database->database_lock);

    sqlite3_reset(database->settings_select_statement);
    sqlite3_bind_text(database->settings_select_statement, 1, key, strlen(key), SQLITE_STATIC);
    int result = sqlite3_step(database->settings_select_statement);

    // If the settings exists update it in the database
    if (result == SQLITE_ROW) {
        sqlite3_reset(database->settings_update_statement);
        sqlite3_bind_text(database->settings_update_statement, 1, value, strlen(value), SQLITE_STATIC);
        sqlite3_bind_text(database->settings_update_statement, 2, key, strlen(key), SQLITE_STATIC);
        if (sqlite3_step(database->settings_update_statement) != SQLITE_DONE) {
            log_error("Can't update setting %s in database", key);
        }
    }

    // Else insert it into the database
    else {
        sqlite3_reset(database->settings_insert_statement);
        sqlite3_bind_text(database->settings_insert_statement, 1, key, strlen(key), SQLITE_STATIC);
        sqlite3_bind_text(database->settings_insert_statement, 2, value, strlen(value), SQLITE_STATIC);
        if (sqlite3_step(database->settings_insert_statement) != SQLITE_DONE) {
            log_error("Can't insert setting %s into database", key);
        }
    }

    mtx_unlock(&database->database_lock);

    database_check_commit(database);
}

void database_settings_set_int(Database* database, char* key, int value) {
    char value_string[128];
    sprintf(value_string, "%d", value);
    database_settings_set_string(database, key, value_string);
}

void database_settings_set_float(Database* database, char* key, float value) {
    char value_string[128];
    sprintf(value_string, "%f", value);
    database_settings_set_string(database, key, value_string);
}

Chunk* database_chunks_get_chunk(Database* database, int chunk_x, int chunk_y, int chunk_z) {
    mtx_lock(&database->database_lock);

    Chunk* chunk;

    // Do a select query to select the chunk from the database
    sqlite3_reset(database->chunks_select_statement);
    sqlite3_bind_int(database->chunks_select_statement, 1, chunk_x);
    sqlite3_bind_int(database->chunks_select_statement, 2, chunk_y);
    sqlite3_bind_int(database->chunks_select_statement, 3, chunk_z);
    if (sqlite3_step(database->chunks_select_statement) == SQLITE_ROW) {
        const uint8_t* compressed_data = sqlite3_column_blob(database->chunks_select_statement, 0);
        uint8_t *chunk_data = chunk_data_decompress((uint8_t*)compressed_data);
        chunk = chunk_new_from_data(chunk_x, chunk_y, chunk_z, chunk_data);
    }
    else {
        chunk = NULL;
    }

    mtx_unlock(&database->database_lock);

    return chunk;
}

void database_chunks_set_chunk(Database* database, Chunk* chunk) {
    mtx_lock(&database->database_lock);

    // Compress chunk data
    uint8_t* compressed_data = chunk_data_compress(chunk->data);
    int compressed_size = (compressed_data[1] << 8) | compressed_data[0];

    // Do a select query to check if the chunk exists in the database
    sqlite3_reset(database->chunks_select_statement);
    sqlite3_bind_int(database->chunks_select_statement, 1, chunk->x);
    sqlite3_bind_int(database->chunks_select_statement, 2, chunk->y);
    sqlite3_bind_int(database->chunks_select_statement, 3, chunk->z);
    int result = sqlite3_step(database->chunks_select_statement);

    // If it exists update the chunk
    if (result == SQLITE_ROW) {
        sqlite3_reset(database->chunks_update_statement);
        sqlite3_bind_blob(database->chunks_update_statement, 1, compressed_data, compressed_size, SQLITE_STATIC);
        sqlite3_bind_int(database->chunks_update_statement, 2, chunk->x);
        sqlite3_bind_int(database->chunks_update_statement, 3, chunk->y);
        sqlite3_bind_int(database->chunks_update_statement, 4, chunk->z);
        if (sqlite3_step(database->chunks_update_statement) != SQLITE_DONE) {
            log_error("Can't update chunk in database");
        }
    }

    // Else insert the chunk into the database
    else {
        sqlite3_reset(database->chunks_insert_statement);
        sqlite3_bind_int(database->chunks_insert_statement, 1, chunk->x);
        sqlite3_bind_int(database->chunks_insert_statement, 2, chunk->y);
        sqlite3_bind_int(database->chunks_insert_statement, 3, chunk->z);
        sqlite3_bind_blob(database->chunks_insert_statement, 4, compressed_data, compressed_size, SQLITE_STATIC);
        if (sqlite3_step(database->chunks_insert_statement) != SQLITE_DONE) {
            log_error("Can't insert chunk into database");
        }
    }

    // Free compressed data
    free(compressed_data);

    mtx_unlock(&database->database_lock);

    database_check_commit(database);
}

void database_commit(Database* database) {
    mtx_lock(&database->database_lock);

    // Commit pending database transaction and start a new own
    char *error_message = NULL;
    if (sqlite3_exec(database->database, "COMMIT;BEGIN", NULL, NULL, &error_message) != SQLITE_OK) {
        log_error("Can't commit database transaction:\n%s", error_message);
    }

    mtx_unlock(&database->database_lock);
}

void database_check_commit(Database* database) {
    // Check if the changes counter is at the commit rate then reset and commit
    if (database->database_changes == DATABASE_COMMIT_RATE) {
        database->database_changes = 0;
        database_commit(database);
    } else {
        database->database_changes++;
    }
}

void database_free(Database* database) {
    // Commit pending transactions
    database_commit(database);

    // Free statements
    sqlite3_finalize(database->settings_select_statement);
    sqlite3_finalize(database->settings_insert_statement);
    sqlite3_finalize(database->settings_update_statement);

    sqlite3_finalize(database->chunks_select_statement);
    sqlite3_finalize(database->chunks_insert_statement);
    sqlite3_finalize(database->chunks_update_statement);

    // Close database connection
    sqlite3_close(database->database);

    // Free database object
    free(database);
}
