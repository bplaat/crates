// PlaatCraft - Database Header

#ifndef DATABASE_H
#define DATABASE_H

#include <sqlite3.h>
#include "tinycthread/tinycthread.h"

typedef struct Database {
    sqlite3* database;
    mtx_t database_lock;
    int database_changes;

    sqlite3_stmt* settings_select_statement;
    sqlite3_stmt* settings_insert_statement;
    sqlite3_stmt* settings_update_statement;

    sqlite3_stmt* chunks_select_statement;
    sqlite3_stmt* chunks_insert_statement;
    sqlite3_stmt* chunks_update_statement;
} Database;

#include "chunk.h" // Fix circle dependancy

Database* database_new(void);


char* database_settings_get_string(Database* database, char* key, char* default_value);

int database_settings_get_int(Database* database, char* key, int default_value);

float database_settings_get_float(Database* database, char* key, float default_value);


void database_settings_set_string(Database* database, char* key, char* value);

void database_settings_set_int(Database* database, char* key, int value);

void database_settings_set_float(Database* database, char* key, float value);


Chunk* database_chunks_get_chunk(Database* database, int chunk_x, int chunk_y, int chunk_z);

void database_chunks_set_chunk(Database* database, Chunk* chunk);


void database_commit(Database* database);

void database_check_commit(Database* database);


void database_free(Database* database);

#endif
