CREATE TABLE persons(
    id BLOB PRIMARY KEY,
    name TEXT NOT NULL,
    age INTEGER NOT NULL,
    relation INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

-- bsqlite:create_fts5_table persons, name
