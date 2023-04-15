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

CREATE TABLE events (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT NULL,
    color TEXT NULL,
    start_date INTEGER NOT NULL,
    end_date INTEGER NOT NULL,
    location_lng REAL NULL,
    location_lat REAL NULL
) STRICT;

-- CREATE TABLE event_guests (
--     id INTEGER PRIMARY KEY NOT NULL,
--     event_id INTEGER NOT NULL,
--     guest_name TEXT NOT NULL,

--     UNIQUE(event_id, guest_name),

--     CONSTRAINT fk_guest_assoc
--         FOREIGN KEY (guest_name)
--         REFERENCES users (username)
--         ON DELETE CASCADE,

--     CONSTRAINT fk_event_id_assoc
--         FOREIGN KEY (event_id)
--         REFERENCES events (id)
--         ON DELETE CASCADE
-- ) STRICT;