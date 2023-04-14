CREATE TABLE users (
    -- "PRIMARY KEY" makes the database optimize this table for accesses via this
    -- column and makes it's content unique (only one user per username).
    --
    -- "NOT NULL" disallows NULL because columns in SQL are nullable by default.
    --
    -- "COLLATE NOCASE" makes this column case insensitive.
    username TEXT PRIMARY KEY NOT NULL COLLATE NOCASE,
    created_at INTEGER NOT NULL -- unix ts
) STRICT;
